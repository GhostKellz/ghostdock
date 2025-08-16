use crate::error::Result;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // User ID
    pub name: String,       // User name
    pub email: String,      // User email
    pub exp: usize,         // Expiration time
    pub iat: usize,         // Issued at
    pub iss: String,        // Issuer
    pub scope: Vec<String>, // Permissions/scopes
}

#[derive(Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub issuer: String,
    pub expiration_hours: u64,
}

impl JwtConfig {
    pub fn new(secret: String) -> Self {
        Self {
            secret,
            issuer: "ghostdock".to_string(),
            expiration_hours: 24,
        }
    }
}

/// Generate a new JWT token for a user
pub fn generate_token(
    user_id: &str,
    name: &str,
    email: &str,
    scopes: Vec<String>,
    config: &JwtConfig,
) -> Result<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let exp = now + (config.expiration_hours * 3600) as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        name: name.to_string(),
        email: email.to_string(),
        exp,
        iat: now,
        iss: config.issuer.clone(),
        scope: scopes,
    };

    let header = Header::new(Algorithm::HS256);
    let encoding_key = EncodingKey::from_secret(config.secret.as_ref());

    encode(&header, &claims, &encoding_key)
        .map_err(|e| crate::error::Error::from(anyhow::anyhow!("JWT encoding failed: {}", e)))
}

/// Validate a JWT token and extract claims
pub fn validate_token(token: &str, config: &JwtConfig) -> Result<Claims> {
    let decoding_key = DecodingKey::from_secret(config.secret.as_ref());
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&[&config.issuer]);

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| {
            crate::error::Error::from(anyhow::anyhow!("JWT validation failed: {}", e))
        })?;

    Ok(token_data.claims)
}

/// Extract token from Authorization header
pub fn extract_token_from_header(auth_header: &str) -> Option<&str> {
    if auth_header.starts_with("Bearer ") {
        Some(&auth_header[7..])
    } else {
        None
    }
}

/// Check if user has required scope/permission
pub fn has_scope(claims: &Claims, required_scope: &str) -> bool {
    claims.scope.contains(&required_scope.to_string()) || 
    claims.scope.contains(&"admin".to_string()) // Admin has all permissions
}

/// Generate scopes based on user role
pub fn generate_scopes_for_role(role: &str) -> Vec<String> {
    match role {
        "admin" => vec![
            "admin".to_string(),
            "registry:read".to_string(),
            "registry:write".to_string(),
            "registry:delete".to_string(),
            "user:manage".to_string(),
            "stack:manage".to_string(),
        ],
        "developer" => vec![
            "registry:read".to_string(),
            "registry:write".to_string(),
            "stack:manage".to_string(),
        ],
        "reader" => vec![
            "registry:read".to_string(),
        ],
        _ => vec!["registry:read".to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_flow() {
        let config = JwtConfig::new("test-secret".to_string());
        let scopes = generate_scopes_for_role("developer");
        
        // Generate token
        let token = generate_token("user123", "Test User", "test@example.com", scopes, &config)
            .expect("Failed to generate token");
        
        // Validate token
        let claims = validate_token(&token, &config)
            .expect("Failed to validate token");
        
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.name, "Test User");
        assert!(has_scope(&claims, "registry:read"));
        assert!(has_scope(&claims, "registry:write"));
        assert!(!has_scope(&claims, "admin"));
    }
}
