use crate::{
    config::OAuthProvider,
    error::{Error, Result},
    models::{LoginRequest, LoginResponse, UserModel},
    server::AppState,
    types::Claims,
    utils::verify_password,
};
use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect},
    Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::Deserialize;
use std::collections::HashMap;

/// Handle user login with username/password
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse> {
    // Find user by username or email
    let user = sqlx::query_as::<_, UserModel>(
        "SELECT * FROM users WHERE username = $1 OR email = $1"
    )
    .bind(&request.username)
    .fetch_optional(&state.database.pool)
    .await?
    .ok_or_else(|| Error::authentication("Invalid username or password"))?;

    // Check if user is active
    if !user.is_active {
        return Err(Error::authentication("Account is disabled"));
    }

    // Verify password
    let password_hash = user.password_hash
        .as_ref()
        .ok_or_else(|| Error::authentication("Password authentication not available"))?;

    if !verify_password(&request.password, password_hash).await? {
        return Err(Error::authentication("Invalid username or password"));
    }

    // Update last login
    sqlx::query("UPDATE users SET last_login = $1 WHERE id = $2")
        .bind(Utc::now())
        .bind(&user.id)
        .execute(&state.database.pool)
        .await?;

    // Generate JWT token
    let expires_at = Utc::now() + Duration::seconds(state.config.auth.jwt_expiration as i64);
    let claims = Claims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        email: user.email.clone(),
        is_admin: user.is_admin,
        exp: expires_at.timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.auth.jwt_secret.as_ref()),
    )?;

    Ok(Json(LoginResponse {
        token,
        user,
        expires_at,
    }))
}

/// Handle user logout
pub async fn logout() -> Result<impl IntoResponse> {
    // In a stateless JWT system, logout is handled client-side
    // In the future, we could implement token blacklisting
    Ok(Json(serde_json::json!({
        "message": "Successfully logged out"
    })))
}

/// OAuth redirect endpoint
pub async fn oauth_redirect(
    State(state): State<AppState>,
    Path(provider): Path<String>,
) -> Result<impl IntoResponse> {
    let oauth_config = match provider.as_str() {
        "google" => state.config.auth.oauth.google.as_ref(),
        "github" => state.config.auth.oauth.github.as_ref(),
        "microsoft" => state.config.auth.oauth.microsoft.as_ref(),
        _ => return Err(Error::bad_request("Unsupported OAuth provider")),
    };

    let oauth_config = oauth_config
        .ok_or_else(|| Error::bad_request("OAuth provider not configured"))?;

    if !oauth_config.enabled {
        return Err(Error::bad_request("OAuth provider is disabled"));
    }

    let client = create_oauth_client(&provider, oauth_config)?;

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .url();

    Ok(Redirect::to(auth_url.as_ref()))
}

/// OAuth callback endpoint
pub async fn oauth_callback(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse> {
    let code = params
        .get("code")
        .ok_or_else(|| Error::authentication("Authorization code not provided"))?;

    let oauth_config = match provider.as_str() {
        "google" => state.config.auth.oauth.google.as_ref(),
        "github" => state.config.auth.oauth.github.as_ref(),
        "microsoft" => state.config.auth.oauth.microsoft.as_ref(),
        _ => return Err(Error::bad_request("Unsupported OAuth provider")),
    };

    let oauth_config = oauth_config
        .ok_or_else(|| Error::bad_request("OAuth provider not configured"))?;

    let client = create_oauth_client(&provider, oauth_config)?;

    // Exchange the code for a token
    let token_result = client
        .exchange_code(AuthorizationCode::new(code.clone()))
        .request_async(oauth2::reqwest::async_http_client)
        .await
        .map_err(|e| Error::authentication(format!("Failed to exchange code for token: {}", e)))?;

    // Get user info from the provider
    let user_info = get_user_info_from_provider(&provider, token_result.access_token().secret()).await?;

    // Create or update user
    let user = create_or_update_oauth_user(&state, &provider, user_info).await?;

    // Update last login
    sqlx::query("UPDATE users SET last_login = $1 WHERE id = $2")
        .bind(Utc::now())
        .bind(&user.id)
        .execute(&state.database.pool)
        .await?;

    // Generate JWT token
    let expires_at = Utc::now() + Duration::seconds(state.config.auth.jwt_expiration as i64);
    let claims = Claims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        email: user.email.clone(),
        is_admin: user.is_admin,
        exp: expires_at.timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.auth.jwt_secret.as_ref()),
    )?;

    // Redirect to frontend with token (you might want to use a different approach)
    Ok(Redirect::to(&format!("/auth/callback?token={}", token)))
}

fn create_oauth_client(provider: &str, config: &OAuthProvider) -> Result<BasicClient> {
    let client_id = ClientId::new(config.client_id.clone());
    let client_secret = ClientSecret::new(config.client_secret.clone());
    let redirect_url = RedirectUrl::new(config.redirect_url.clone())
        .map_err(|e| Error::internal(format!("Invalid redirect URL: {}", e)))?;

    let (auth_url, token_url) = match provider {
        "google" => (
            "https://accounts.google.com/o/oauth2/auth",
            "https://oauth2.googleapis.com/token",
        ),
        "github" => (
            "https://github.com/login/oauth/authorize",
            "https://github.com/login/oauth/access_token",
        ),
        "microsoft" => (
            "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
            "https://login.microsoftonline.com/common/oauth2/v2.0/token",
        ),
        _ => return Err(Error::bad_request("Unsupported OAuth provider")),
    };

    let auth_url = AuthUrl::new(auth_url.to_string())
        .map_err(|e| Error::internal(format!("Invalid auth URL: {}", e)))?;
    let token_url = TokenUrl::new(token_url.to_string())
        .map_err(|e| Error::internal(format!("Invalid token URL: {}", e)))?;

    Ok(BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(redirect_url))
}

#[derive(Deserialize)]
struct OAuthUserInfo {
    id: String,
    email: String,
    name: Option<String>,
    login: Option<String>, // GitHub username
    picture: Option<String>, // Google avatar
    avatar_url: Option<String>, // GitHub avatar
}

async fn get_user_info_from_provider(provider: &str, access_token: &str) -> Result<OAuthUserInfo> {
    let client = reqwest::Client::new();
    let user_info_url = match provider {
        "google" => "https://www.googleapis.com/oauth2/v2/userinfo",
        "github" => "https://api.github.com/user",
        "microsoft" => "https://graph.microsoft.com/v1.0/me",
        _ => return Err(Error::bad_request("Unsupported OAuth provider")),
    };

    let response = client
        .get(user_info_url)
        .bearer_auth(access_token)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(Error::authentication("Failed to get user info from OAuth provider"));
    }

    let user_info: OAuthUserInfo = response.json().await?;
    Ok(user_info)
}

async fn create_or_update_oauth_user(
    state: &AppState,
    provider: &str,
    user_info: OAuthUserInfo,
) -> Result<UserModel> {
    // Check if user exists with this provider ID
    if let Some(existing_user) = sqlx::query_as::<_, UserModel>(
        "SELECT * FROM users WHERE provider = $1 AND provider_id = $2"
    )
    .bind(provider)
    .bind(&user_info.id)
    .fetch_optional(&state.database.pool)
    .await?
    {
        return Ok(existing_user);
    }

    // Check if user exists with this email
    if let Some(existing_user) = sqlx::query_as::<_, UserModel>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(&user_info.email)
    .fetch_optional(&state.database.pool)
    .await?
    {
        // Update existing user with OAuth info
        let updated_user = sqlx::query_as::<_, UserModel>(
            "UPDATE users SET provider = $1, provider_id = $2, updated_at = $3 WHERE id = $4 RETURNING *"
        )
        .bind(provider)
        .bind(&user_info.id)
        .bind(Utc::now())
        .bind(&existing_user.id)
        .fetch_one(&state.database.pool)
        .await?;

        return Ok(updated_user);
    }

    // Create new user
    let username = user_info.login.clone()
        .or_else(|| user_info.name.clone())
        .unwrap_or_else(|| format!("user_{}", &user_info.id[..8]));

    let avatar_url = user_info.picture.or(user_info.avatar_url);

    let new_user = sqlx::query_as::<_, UserModel>(
        r#"
        INSERT INTO users (id, username, email, full_name, avatar_url, provider, provider_id, is_admin, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING *
        "#
    )
    .bind(uuid::Uuid::new_v4())
    .bind(&username)
    .bind(&user_info.email)
    .bind(&user_info.name)
    .bind(&avatar_url)
    .bind(provider)
    .bind(&user_info.id)
    .bind(false) // is_admin
    .bind(true)  // is_active
    .bind(Utc::now())
    .bind(Utc::now())
    .fetch_one(&state.database.pool)
    .await?;

    Ok(new_user)
}
