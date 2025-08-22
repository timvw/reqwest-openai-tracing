//! # reqwest-openai-tracing
//!
//! OpenTelemetry tracing middleware for OpenAI API calls made with reqwest.
//!
//! This library provides automatic tracing for OpenAI API calls, with support for:
//! - Automatic span creation for chat completions, embeddings, and other OpenAI operations
//! - Token usage tracking
//! - Langfuse integration via OpenTelemetry
//! - Customizable trace attributes (session_id, user_id, tags, metadata)
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use reqwest_openai_tracing::{OpenAITracingMiddleware, HttpClientWithMiddleware, context};
//! use async_openai::{config::AzureConfig, Client};
//! use reqwest_middleware::ClientBuilder;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create reqwest client with tracing middleware
//! let reqwest_client = reqwest::Client::new();
//! let middleware_client = ClientBuilder::new(reqwest_client)
//!     .with(OpenAITracingMiddleware::new())
//!     .build();
//!
//! // Wrap for async-openai compatibility
//! let http_client = HttpClientWithMiddleware::new(middleware_client);
//!
//! // Configure OpenAI client
//! let config = AzureConfig::new()
//!     .with_api_base("https://your-endpoint.openai.azure.com")
//!     .with_api_key("your-api-key")
//!     .with_deployment_id("gpt-4");
//!
//! let client = Client::build(http_client, config, Default::default());
//!
//! // Set context attributes (optional)
//! context::set_session_id("session-123");
//! context::set_user_id("user-456");
//!
//! // Make API calls - they will be automatically traced
//! # Ok(())
//! # }
//! ```

mod attributes;
mod context;
mod http_client;
mod langfuse;
mod middleware;

// Re-export main types
pub use attributes::{LangfuseAttributes, ObservationAttributesBuilder, TraceAttributesBuilder};
pub use context::{
    add_tags, apply_context, set_session_id, set_user_id, LangfuseContext, LangfuseContextBuilder,
    GLOBAL_CONTEXT,
};
pub use http_client::HttpClientWithMiddleware;
pub use middleware::OpenAITracingMiddleware;

// Re-export context module for convenient access
pub mod langfuse_context {
    pub use crate::context::*;
}

// Re-export langfuse utilities
pub use langfuse::{
    build_langfuse_auth_header, build_langfuse_auth_header_from_env,
    build_langfuse_otlp_endpoint_from_env, build_otlp_endpoint,
};
