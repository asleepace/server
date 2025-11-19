use std::path::{Path, PathBuf};
use std::io::{Error, ErrorKind};

#[derive(Debug)]
pub enum SecurityError {
    PathTraversal,
    InvalidCharacters,
    PathTooLong,
    OutOfBounds,
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SecurityError::PathTraversal => write!(f, "Path traversal attempt detected"),
            SecurityError::InvalidCharacters => write!(f, "Invalid characters in path"),
            SecurityError::PathTooLong => write!(f, "Path exceeds maximum length"),
            SecurityError::OutOfBounds => write!(f, "Path outside allowed directory"),
        }
    }
}

impl std::error::Error for SecurityError {}

impl From<SecurityError> for Error {
    fn from(err: SecurityError) -> Error {
        Error::new(ErrorKind::PermissionDenied, err)
    }
}

/// Sanitizes and validates a URL path for security
/// Prevents path traversal attacks and validates allowed characters
pub fn sanitize_path(input: &str) -> Result<String, SecurityError> {
    // Check path length
    if input.len() > 255 {
        return Err(SecurityError::PathTooLong);
    }

    // Remove leading/trailing slashes and whitespace (normal for HTTP URLs)
    let path = input.trim_matches(&[' ', '/', '\\'][..]);

    // Check for absolute paths after trimming (Windows drive letters)
    if path.len() > 1 && path.chars().nth(1) == Some(':') {
        eprintln!("[security] Windows absolute path blocked: {}", input);
        return Err(SecurityError::PathTraversal);
    }

    // Check for path traversal attempts
    if path.contains("..") || path.contains("~") {
        eprintln!("[security] Path traversal attempt blocked: {}", input);
        return Err(SecurityError::PathTraversal);
    }

    // Validate allowed characters: alphanumeric, hyphens, underscores, dots, forward slashes
    let allowed_chars = |c: char| {
        c.is_alphanumeric() || matches!(c, '-' | '_' | '.' | '/')
    };

    if !path.chars().all(allowed_chars) {
        eprintln!("[security] Invalid characters in path: {}", input);
        return Err(SecurityError::InvalidCharacters);
    }

    // Normalize path separators to forward slashes
    let normalized = path.replace('\\', "/");

    // Remove any double slashes
    let clean_path = normalized
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<&str>>()
        .join("/");

    Ok(clean_path)
}

/// Validates that a resolved path is within the allowed base directory
pub fn validate_path_bounds(resolved_path: &Path, base_dir: &Path) -> Result<(), SecurityError> {
    // Canonicalize both paths to resolve any symlinks and relative components
    let canonical_resolved = match resolved_path.canonicalize() {
        Ok(path) => path,
        Err(_) => {
            // If canonicalize fails, the path probably doesn't exist, which is OK
            // We'll check if the parent directory structure is valid
            match resolved_path.parent() {
                Some(parent) => match parent.canonicalize() {
                    Ok(parent_canonical) => parent_canonical.join(
                        resolved_path.file_name().unwrap_or_default()
                    ),
                    Err(_) => return Err(SecurityError::OutOfBounds),
                },
                None => return Err(SecurityError::OutOfBounds),
            }
        }
    };

    let canonical_base = match base_dir.canonicalize() {
        Ok(path) => path,
        Err(_) => return Err(SecurityError::OutOfBounds),
    };

    // Check if the resolved path starts with the base directory
    if !canonical_resolved.starts_with(&canonical_base) {
        eprintln!(
            "[security] Path outside bounds: {:?} not within {:?}",
            canonical_resolved, canonical_base
        );
        return Err(SecurityError::OutOfBounds);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_path_normal() {
        assert_eq!(sanitize_path("/normal/path").unwrap(), "normal/path");
        assert_eq!(sanitize_path("file.html").unwrap(), "file.html");
        assert_eq!(sanitize_path("folder/file.html").unwrap(), "folder/file.html");
    }

    #[test]
    fn test_sanitize_path_traversal() {
        assert!(sanitize_path("../etc/passwd").is_err());
        assert!(sanitize_path("folder/../../../etc/passwd").is_err());
        assert!(sanitize_path("~/.ssh/id_rsa").is_err());
    }

    #[test]
    fn test_sanitize_path_invalid_chars() {
        assert!(sanitize_path("file<script>").is_err());
        assert!(sanitize_path("file|rm -rf").is_err());
        assert!(sanitize_path("file&whoami").is_err());
    }

    #[test]
    fn test_sanitize_path_absolute() {
        // Windows absolute paths should be blocked
        assert!(sanitize_path("C:\\Windows\\System32").is_err());
        assert!(sanitize_path("D:/Program Files").is_err());
        
        // Unix-style leading slashes are normal HTTP paths and should be allowed (after trimming)
        assert_eq!(sanitize_path("/normal/path").unwrap(), "normal/path");
        assert_eq!(sanitize_path("/file.html").unwrap(), "file.html");
    }
}
