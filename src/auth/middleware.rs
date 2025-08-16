use axum::{
    extract::{Request, State, FromRequestParts},
    http::{HeaderMap, StatusCode, request::Parts},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use async_trait::async_trait;

use crate::auth::jwt::{validate_token, extract_token_from_header, has_scope, Claims, JwtConfig};

/// Authentication state passed to middleware
#[derive(Clone)]
pub struct AuthState {
    pub jwt_config: JwtConfig,
    pub require_auth: bool,
}

/// User information extracted from JWT
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: String,
    pub name: String,
    pub email: String,
    pub scopes: Vec<String>,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract Authorization header
        let authorization = parts
            .headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        // Extract token from "Bearer <token>" format
        let token = extract_token_from_header(authorization)
            .ok_or(StatusCode::UNAUTHORIZED)?;

        // For now, create a default JWT config
        // TODO: This should come from app state or configuration
        let jwt_config = JwtConfig::new(
            std::env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret".to_string())
        );

        // Validate token and extract claims
        let claims = validate_token(&token, &jwt_config)
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(AuthenticatedUser::from(claims))
    }
}

impl From<Claims> for AuthenticatedUser {
    fn from(claims: Claims) -> Self {
        Self {
            id: claims.sub,
            name: claims.name,
            email: claims.email,
            scopes: claims.scope,
        }
    }
}

/// Authentication middleware that validates JWT tokens
pub async fn auth_middleware(
    State(auth_state): State<AuthState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip auth for health checks and some public endpoints
    let path = request.uri().path();
    if is_public_endpoint(path) {
        return Ok(next.run(request).await);
    }

    // Extract authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok());

    if let Some(auth_header) = auth_header {
        if let Some(token) = extract_token_from_header(auth_header) {
            match validate_token(token, &auth_state.jwt_config) {
                Ok(claims) => {
                    // Add user info to request extensions
                    let user = AuthenticatedUser::from(claims);
                    request.extensions_mut().insert(user);
                    return Ok(next.run(request).await);
                }
                Err(_) => {
                    return Err(StatusCode::UNAUTHORIZED);
                }
            }
        }
    }

    // Check if auth is required
    if auth_state.require_auth || requires_auth(path) {
        Err(StatusCode::UNAUTHORIZED)
    } else {
        Ok(next.run(request).await)
    }
}

/// Scope-based authorization middleware
pub fn require_scope(required_scope: &'static str) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
    move |request: Request, next: Next| {
        let scope = required_scope;
        Box::pin(async move {
            // Get authenticated user from request extensions
            if let Some(user) = request.extensions().get::<AuthenticatedUser>() {
                if user.scopes.contains(&scope.to_string()) || user.scopes.contains(&"admin".to_string()) {
                    Ok(next.run(request).await)
                } else {
                    Err(StatusCode::FORBIDDEN)
                }
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        })
    }
}

/// Check if endpoint is public (doesn't require authentication)
fn is_public_endpoint(path: &str) -> bool {
    matches!(
        path,
        "/health" | 
        "/metrics" | 
        "/" | 
        "/auth/login" | 
        "/auth/oauth/google" | 
        "/auth/oauth/github" | 
        "/auth/oauth/microsoft" |
        "/auth/oauth/google/callback" |
        "/auth/oauth/github/callback" |
        "/auth/oauth/microsoft/callback" |
        "/v2/" // Docker registry base endpoint
    )
}

/// Check if endpoint requires authentication
fn requires_auth(path: &str) -> bool {
    // Registry endpoints that require auth
    path.starts_with("/v2/") && path != "/v2/" ||
    path.starts_with("/api/") ||
    path.starts_with("/dashboard") ||
    path.starts_with("/repositories") ||
    path.starts_with("/users") ||
    path.starts_with("/settings") ||
    path.starts_with("/stacks") // New stack management endpoints
}

/// Helper to extract authenticated user from request
pub fn get_authenticated_user(request: &Request) -> Option<&AuthenticatedUser> {
    request.extensions().get::<AuthenticatedUser>()
}

/// Create auth state for middleware
pub fn create_auth_state(jwt_secret: String, require_auth: bool) -> AuthState {
    AuthState {
        jwt_config: JwtConfig::new(jwt_secret),
        require_auth,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_endpoints() {
        assert!(is_public_endpoint("/health"));
        assert!(is_public_endpoint("/"));
        assert!(is_public_endpoint("/auth/login"));
        assert!(!is_public_endpoint("/dashboard"));
        assert!(!is_public_endpoint("/repositories"));
    }

    #[test]
    fn test_auth_required_endpoints() {
        assert!(requires_auth("/v2/myrepo/manifests/latest"));
        assert!(requires_auth("/api/stacks"));
        assert!(requires_auth("/dashboard"));
        assert!(!requires_auth("/v2/"));
        assert!(!requires_auth("/health"));
    }
}
