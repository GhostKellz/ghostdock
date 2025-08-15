use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Authentication error: {message}")]
    Authentication { message: String },

    #[error("Authorization error: {message}")]
    Authorization { message: String },

    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Registry error: {message}")]
    Registry { message: String },

    #[error("Storage error: {message}")]
    Storage { message: String },

    #[error("Manifest error: {message}")]
    Manifest { message: String },

    #[error("Blob error: {message}")]
    Blob { message: String },

    #[error("Not found: {resource}")]
    NotFound { resource: String },

    #[error("Conflict: {message}")]
    Conflict { message: String },

    #[error("Internal server error: {message}")]
    Internal { message: String },

    #[error("Bad request: {message}")]
    BadRequest { message: String },

    #[error("Service unavailable: {message}")]
    ServiceUnavailable { message: String },

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Generic error: {0}")]
    Generic(#[from] anyhow::Error),
}

impl Error {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Error::Authentication { .. } => StatusCode::UNAUTHORIZED,
            Error::Authorization { .. } => StatusCode::FORBIDDEN,
            Error::Validation { .. } => StatusCode::BAD_REQUEST,
            Error::BadRequest { .. } => StatusCode::BAD_REQUEST,
            Error::NotFound { .. } => StatusCode::NOT_FOUND,
            Error::Conflict { .. } => StatusCode::CONFLICT,
            Error::ServiceUnavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
            Error::Registry { .. } => StatusCode::BAD_REQUEST,
            Error::Storage { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Manifest { .. } => StatusCode::BAD_REQUEST,
            Error::Blob { .. } => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            Error::Database(_) => "DATABASE_ERROR",
            Error::Io(_) => "IO_ERROR",
            Error::Serialization(_) => "SERIALIZATION_ERROR",
            Error::Config(_) => "CONFIG_ERROR",
            Error::Authentication { .. } => "AUTHENTICATION_ERROR",
            Error::Authorization { .. } => "AUTHORIZATION_ERROR",
            Error::Validation { .. } => "VALIDATION_ERROR",
            Error::Registry { .. } => "REGISTRY_ERROR",
            Error::Storage { .. } => "STORAGE_ERROR",
            Error::Manifest { .. } => "MANIFEST_ERROR",
            Error::Blob { .. } => "BLOB_ERROR",
            Error::NotFound { .. } => "NOT_FOUND",
            Error::Conflict { .. } => "CONFLICT",
            Error::Internal { .. } => "INTERNAL_ERROR",
            Error::BadRequest { .. } => "BAD_REQUEST",
            Error::ServiceUnavailable { .. } => "SERVICE_UNAVAILABLE",
            Error::Jwt(_) => "JWT_ERROR",
            Error::HttpClient(_) => "HTTP_CLIENT_ERROR",
            Error::Toml(_) => "TOML_ERROR",
            Error::Generic(_) => "GENERIC_ERROR",
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = json!({
            "error": {
                "code": self.error_code(),
                "message": self.to_string()
            }
        });

        (status, Json(error_response)).into_response()
    }
}

// Helper functions for creating common errors
impl Error {
    pub fn authentication<S: Into<String>>(message: S) -> Self {
        Self::Authentication {
            message: message.into(),
        }
    }

    pub fn authorization<S: Into<String>>(message: S) -> Self {
        Self::Authorization {
            message: message.into(),
        }
    }

    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    pub fn registry<S: Into<String>>(message: S) -> Self {
        Self::Registry {
            message: message.into(),
        }
    }

    pub fn storage<S: Into<String>>(message: S) -> Self {
        Self::Storage {
            message: message.into(),
        }
    }

    pub fn not_found<S: Into<String>>(resource: S) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    pub fn conflict<S: Into<String>>(message: S) -> Self {
        Self::Conflict {
            message: message.into(),
        }
    }

    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    pub fn bad_request<S: Into<String>>(message: S) -> Self {
        Self::BadRequest {
            message: message.into(),
        }
    }
}
