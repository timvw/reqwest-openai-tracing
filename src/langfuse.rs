//! Langfuse integration utilities

use base64::{engine::general_purpose::STANDARD, Engine};
use std::env;

/// Builds a Langfuse authentication header value from public and secret keys.
///
/// This function concatenates the public and secret keys with a colon separator,
/// encodes them in base64, and returns the complete "Basic {auth}" string.
///
/// # Arguments
///
/// * `public_key` - The Langfuse public key
/// * `secret_key` - The Langfuse secret key
///
/// # Returns
///
/// Returns the complete authentication header value "Basic {base64_encoded}".
///
/// # Example
///
/// ```rust,no_run
/// use reqwest_openai_tracing::build_langfuse_auth_header;
///
/// let auth = build_langfuse_auth_header(
///     "pk-lf-1234567890",
///     "sk-lf-1234567890"
/// );
/// // auth = "Basic cGstbGYtMTIzNDU2Nzg5MDpzay1sZi0xMjM0NTY3ODkw"
/// ```
pub fn build_langfuse_auth_header(public_key: &str, secret_key: &str) -> String {
    let auth_string = format!("{}:{}", public_key, secret_key);
    let encoded = STANDARD.encode(auth_string.as_bytes());
    format!("Basic {}", encoded)
}

/// Builds a Langfuse authentication header value from environment variables.
///
/// This function reads the LANGFUSE_PUBLIC_KEY and LANGFUSE_SECRET_KEY environment
/// variables and creates the complete authentication header value.
///
/// # Returns
///
/// Returns a Result containing the complete authentication header value "Basic {base64_encoded}",
/// or an error if environment variables are missing.
///
/// # Example
///
/// ```rust,no_run
/// use reqwest_openai_tracing::build_langfuse_auth_header_from_env;
///
/// // Using environment variables (LANGFUSE_PUBLIC_KEY and LANGFUSE_SECRET_KEY)
/// let auth = build_langfuse_auth_header_from_env().unwrap();
/// // auth = "Basic cGstbGYtMTIzNDU2Nzg5MDpzay1sZi0xMjM0NTY3ODkw"
/// ```
pub fn build_langfuse_auth_header_from_env() -> Result<String, Box<dyn std::error::Error>> {
    let public_key = env::var("LANGFUSE_PUBLIC_KEY")
        .map_err(|_| "Missing LANGFUSE_PUBLIC_KEY environment variable")?;

    let secret_key = env::var("LANGFUSE_SECRET_KEY")
        .map_err(|_| "Missing LANGFUSE_SECRET_KEY environment variable")?;

    Ok(build_langfuse_auth_header(&public_key, &secret_key))
}

/// Builds the Langfuse OTLP endpoint URL by appending the API path.
///
/// This function takes a base URL and appends "/api/public/otel" to create
/// the full OTLP endpoint URL for Langfuse.
///
/// # Arguments
///
/// * `base_url` - The base Langfuse URL (e.g., "https://cloud.langfuse.com")
///
/// # Returns
///
/// Returns the complete OTLP endpoint URL.
///
/// # Example
///
/// ```rust,no_run
/// use reqwest_openai_tracing::build_otlp_endpoint;
///
/// let endpoint = build_otlp_endpoint("https://cloud.langfuse.com");
/// assert_eq!(endpoint, "https://cloud.langfuse.com/api/public/otel");
/// ```
pub fn build_otlp_endpoint(base_url: &str) -> String {
    let url = base_url.trim_end_matches('/');
    format!("{}/api/public/otel", url)
}

/// Builds the Langfuse OTLP endpoint URL from environment variable.
///
/// This function reads the LANGFUSE_HOST environment variable and creates
/// the complete OTLP endpoint URL by appending "/api/public/otel".
///
/// # Returns
///
/// Returns a Result containing the complete OTLP endpoint URL,
/// or an error if the LANGFUSE_HOST environment variable is missing.
///
/// # Example
///
/// ```rust,no_run
/// use reqwest_openai_tracing::build_langfuse_otlp_endpoint_from_env;
///
/// // Using LANGFUSE_HOST environment variable
/// let endpoint = build_langfuse_otlp_endpoint_from_env().unwrap();
/// std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", endpoint);
/// ```
pub fn build_langfuse_otlp_endpoint_from_env() -> Result<String, Box<dyn std::error::Error>> {
    let base_url =
        env::var("LANGFUSE_HOST").map_err(|_| "Missing LANGFUSE_HOST environment variable")?;

    Ok(build_otlp_endpoint(&base_url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_langfuse_auth_header() {
        let auth = build_langfuse_auth_header("pk-lf-test", "sk-lf-secret");

        // The expected value is "Basic " + base64("pk-lf-test:sk-lf-secret")
        let expected = format!("Basic {}", STANDARD.encode("pk-lf-test:sk-lf-secret"));
        assert_eq!(auth, expected);
    }

    #[test]
    fn test_build_langfuse_auth_header_from_env() {
        // Set env vars for this test
        env::set_var("LANGFUSE_PUBLIC_KEY", "pk-env-test");
        env::set_var("LANGFUSE_SECRET_KEY", "sk-env-secret");

        let auth = build_langfuse_auth_header_from_env().unwrap();
        let expected = format!("Basic {}", STANDARD.encode("pk-env-test:sk-env-secret"));
        assert_eq!(auth, expected);
    }

    #[test]
    fn test_missing_env_keys() {
        // Clear env vars for this test
        env::remove_var("LANGFUSE_PUBLIC_KEY");
        env::remove_var("LANGFUSE_SECRET_KEY");

        let result = build_langfuse_auth_header_from_env();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing LANGFUSE_PUBLIC_KEY"));
    }

    #[test]
    fn test_build_otlp_endpoint() {
        // Test with URL without trailing slash
        let endpoint = build_otlp_endpoint("https://cloud.langfuse.com");
        assert_eq!(endpoint, "https://cloud.langfuse.com/api/public/otel");

        // Test with URL with trailing slash
        let endpoint = build_otlp_endpoint("https://cloud.langfuse.com/");
        assert_eq!(endpoint, "https://cloud.langfuse.com/api/public/otel");

        // Test with US region URL
        let endpoint = build_otlp_endpoint("https://us.cloud.langfuse.com");
        assert_eq!(endpoint, "https://us.cloud.langfuse.com/api/public/otel");
    }

    #[test]
    fn test_build_langfuse_otlp_endpoint_from_env() {
        // Set env var for this test
        env::set_var("LANGFUSE_HOST", "https://cloud.langfuse.com");

        let endpoint = build_langfuse_otlp_endpoint_from_env().unwrap();
        assert_eq!(endpoint, "https://cloud.langfuse.com/api/public/otel");

        // Test with trailing slash in env var
        env::set_var("LANGFUSE_HOST", "https://cloud.langfuse.com/");
        let endpoint = build_langfuse_otlp_endpoint_from_env().unwrap();
        assert_eq!(endpoint, "https://cloud.langfuse.com/api/public/otel");
    }

    #[test]
    fn test_missing_langfuse_host() {
        // Clear env var for this test
        env::remove_var("LANGFUSE_HOST");

        let result = build_langfuse_otlp_endpoint_from_env();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing LANGFUSE_HOST"));
    }
}
