use crate::error::{Error, Result};
use regex::Regex;
use sha2::{Sha256, Digest};

/// Validate repository name according to Docker registry specification
pub fn validate_repository_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(Error::bad_request("Repository name cannot be empty"));
    }
    
    if name.len() > 255 {
        return Err(Error::bad_request("Repository name too long"));
    }
    
    // Repository name regex: lowercase letters, numbers, and separators
    let repo_regex = Regex::new(r"^[a-z0-9]+(?:[._-][a-z0-9]+)*(?:/[a-z0-9]+(?:[._-][a-z0-9]+)*)*$")
        .map_err(|_| Error::internal("Invalid regex"))?;
    
    if !repo_regex.is_match(name) {
        return Err(Error::bad_request(format!(
            "Invalid repository name '{}': must contain only lowercase letters, numbers, and separators",
            name
        )));
    }
    
    Ok(())
}

/// Validate tag name according to Docker registry specification
pub fn validate_tag_name(tag: &str) -> Result<()> {
    if tag.is_empty() {
        return Err(Error::bad_request("Tag name cannot be empty"));
    }
    
    if tag.len() > 128 {
        return Err(Error::bad_request("Tag name too long"));
    }
    
    // Tag name regex: alphanumeric characters, underscores, periods, and dashes
    let tag_regex = Regex::new(r"^[a-zA-Z0-9._-]+$")
        .map_err(|_| Error::internal("Invalid regex"))?;
    
    if !tag_regex.is_match(tag) {
        return Err(Error::bad_request(format!(
            "Invalid tag name '{}': must contain only alphanumeric characters, underscores, periods, and dashes",
            tag
        )));
    }
    
    Ok(())
}

/// Validate digest format (sha256:hex)
pub fn validate_digest(digest: &str) -> Result<()> {
    if !digest.starts_with("sha256:") {
        return Err(Error::bad_request("Digest must start with 'sha256:'"));
    }
    
    let hash_part = &digest[7..]; // Remove "sha256:" prefix
    
    if hash_part.len() != 64 {
        return Err(Error::bad_request("Invalid digest: hash must be 64 characters"));
    }
    
    // Validate hex characters
    if !hash_part.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(Error::bad_request("Invalid digest: hash must contain only hexadecimal characters"));
    }
    
    Ok(())
}

/// Calculate SHA256 digest of data
pub fn sha256_digest(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("sha256:{:x}", result)
}

/// Generate a random UUID string
pub fn generate_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Parse content range header
pub fn parse_content_range(range: &str) -> Result<(u64, u64)> {
    // Expected format: "bytes=start-end" or "start-end"
    let range = range.strip_prefix("bytes=").unwrap_or(range);
    
    let parts: Vec<&str> = range.split('-').collect();
    if parts.len() != 2 {
        return Err(Error::bad_request("Invalid range format"));
    }
    
    let start = parts[0].parse::<u64>()
        .map_err(|_| Error::bad_request("Invalid range start"))?;
    
    let end = if parts[1].is_empty() {
        u64::MAX
    } else {
        parts[1].parse::<u64>()
            .map_err(|_| Error::bad_request("Invalid range end"))?
    };
    
    if start > end {
        return Err(Error::bad_request("Range start cannot be greater than end"));
    }
    
    Ok((start, end))
}

/// Format content range header
pub fn format_content_range(start: u64, end: u64, total: Option<u64>) -> String {
    match total {
        Some(total) => format!("bytes {}-{}/{}", start, end, total),
        None => format!("bytes {}-{}/*", start, end),
    }
}

/// Extract media type from manifest content
pub fn extract_media_type(manifest_content: &str) -> Result<String> {
    let manifest: serde_json::Value = serde_json::from_str(manifest_content)
        .map_err(|_| Error::bad_request("Invalid JSON manifest"))?;
    
    Ok(manifest.get("mediaType")
        .and_then(|v| v.as_str())
        .unwrap_or("application/vnd.docker.distribution.manifest.v2+json")
        .to_string())
}

/// Check if a string is a valid digest format
pub fn is_digest(reference: &str) -> bool {
    reference.starts_with("sha256:") && reference.len() == 71
}

/// Verify a password against a hash
pub async fn verify_password(password: &str, hash: &str) -> crate::error::Result<bool> {
    let password = password.to_string();
    let hash = hash.to_string();
    
    let result = tokio::task::spawn_blocking(move || {
        bcrypt::verify(&password, &hash)
    }).await
    .map_err(|_| crate::error::Error::from(anyhow::anyhow!("Failed to spawn blocking task")))?
    .map_err(|_| crate::error::Error::from(anyhow::anyhow!("Password verification failed")))?;
    
    Ok(result)
}
