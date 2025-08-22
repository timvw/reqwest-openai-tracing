//! Example demonstrating Langfuse authentication setup using environment variables

use dotenv::dotenv;
use reqwest_openai_tracing::{
    build_langfuse_auth_header_from_env, build_langfuse_otlp_endpoint_from_env,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables from .env file
    dotenv().ok();

    // These helper functions expect the following environment variables:
    // - LANGFUSE_HOST: The base URL of your Langfuse instance (e.g., https://cloud.langfuse.com)
    // - LANGFUSE_PUBLIC_KEY: Your Langfuse public key (starts with pk-lf-)
    // - LANGFUSE_SECRET_KEY: Your Langfuse secret key (starts with sk-lf-)

    // Build the OTLP endpoint from LANGFUSE_HOST
    let endpoint = build_langfuse_otlp_endpoint_from_env()?;
    println!("OTLP Endpoint: {}", endpoint);

    // Build the authorization header from LANGFUSE_PUBLIC_KEY and LANGFUSE_SECRET_KEY
    let auth_header = build_langfuse_auth_header_from_env()?;
    println!("Authorization Header: {}", auth_header);

    // These values can then be used directly in your OpenTelemetry configuration
    // as shown in the with_langfuse.rs example

    println!("\nYou can now use these values to configure OpenTelemetry:");
    println!("export OTEL_EXPORTER_OTLP_ENDPOINT=\"{}\"", endpoint);
    println!(
        "export OTEL_EXPORTER_OTLP_HEADERS=\"Authorization={}\"",
        auth_header
    );

    Ok(())
}
