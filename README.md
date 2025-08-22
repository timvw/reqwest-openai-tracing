# reqwest-openai-tracing

OpenTelemetry tracing middleware for OpenAI API calls made with reqwest.

[![Crates.io](https://img.shields.io/crates/v/reqwest-openai-tracing.svg)](https://crates.io/crates/reqwest-openai-tracing)
[![Documentation](https://docs.rs/reqwest-openai-tracing/badge.svg)](https://docs.rs/reqwest-openai-tracing)
[![License](https://img.shields.io/crates/l/reqwest-openai-tracing.svg)](https://github.com/timvw/reqwest-openai-tracing#license)

## Features

- 🔍 **Automatic Tracing**: Automatically creates OpenTelemetry spans for OpenAI API calls following [OpenTelemetry GenAI semantic conventions](https://opentelemetry.io/docs/specs/semconv/gen-ai/)
- 📊 **Token Usage Tracking**: Records prompt and completion token usage in span attributes
- 🏷️ **Langfuse Integration**: Seamlessly integrates with [Langfuse via OpenTelemetry](https://langfuse.com/integrations/native/opentelemetry)
- 🎯 **Context Attributes**: Set session IDs, user IDs, tags, and metadata for traces
- 🚀 **async-openai Compatible**: Works with the async-openai library via HttpClient trait
- 🔧 **Flexible**: Works with any OpenTelemetry backend (Jaeger, Zipkin, Langfuse, etc.)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
# Git dependency required until async-openai publishes HttpClient trait to crates.io
# Use a specific tag/release for stability
reqwest-openai-tracing = { git = "https://github.com/timvw/reqwest-openai-tracing.git", tag = "v0.1.0" }
async-openai = { git = "https://github.com/timvw/async-openai.git", rev = "baadc6a" }
reqwest-middleware = "0.4"
```

**Note:** This crate cannot be published to crates.io yet because it depends on `async-openai` via git (specifically a fork that includes the `HttpClient` trait). Once the `HttpClient` trait is merged upstream and published to crates.io, this crate will also be published there.

## Quick Start

```rust
use reqwest_openai_tracing::{OpenAITracingMiddleware, HttpClientWithMiddleware};
use async_openai::{config::AzureConfig, Client};
use reqwest_middleware::ClientBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create reqwest client with tracing middleware
    let reqwest_client = reqwest::Client::new();
    let middleware_client = ClientBuilder::new(reqwest_client)
        .with(OpenAITracingMiddleware::new())
        .build();

    // Wrap for async-openai compatibility
    let http_client = HttpClientWithMiddleware::new(middleware_client);

    // Configure OpenAI client
    let config = AzureConfig::new()
        .with_api_base("https://your-endpoint.openai.azure.com")
        .with_api_key("your-api-key")
        .with_deployment_id("gpt-4");

    let client = Client::build(http_client, config, Default::default());

    // Make API calls - they will be automatically traced!
    Ok(())
}
```

## Langfuse Integration

This library provides helper functions to simplify [Langfuse's OpenTelemetry integration](https://langfuse.com/integrations/native/opentelemetry).

### Configuration

Setup OpenTelemetry with Langfuse:

```rust
use reqwest_openai_tracing::{
    build_langfuse_auth_header_from_env,
    build_langfuse_otlp_endpoint_from_env,
};
use opentelemetry_otlp::{WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::trace::TracerProvider;
use std::collections::HashMap;

// Build endpoint and auth headers from environment variables
let endpoint = build_langfuse_otlp_endpoint_from_env()?;
let auth_header = build_langfuse_auth_header_from_env()?;

// Create headers HashMap for OpenTelemetry
let mut headers = HashMap::new();
headers.insert("Authorization".to_string(), auth_header);

// Setup OpenTelemetry with Langfuse
let exporter = opentelemetry_otlp::SpanExporter::builder()
    .with_http()
    .with_endpoint(endpoint)
    .with_headers(headers)
    .build()?;

let tracer_provider = TracerProvider::builder()
    .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
    .with_resource(opentelemetry_sdk::Resource::new(vec![
        opentelemetry::KeyValue::new("service.name", "your-service-name"),
    ]))
    .build();
```

### Setting Context Attributes

You can add context to your traces for better organization in Langfuse:

```rust
use reqwest_openai_tracing::langfuse_context;

// Set session and user IDs
langfuse_context::set_session_id("session-123");
langfuse_context::set_user_id("user-456");

// Add tags for filtering
langfuse_context::add_tags(vec!["production".to_string(), "v1.0".to_string()]);

// Add custom metadata
langfuse_context::GLOBAL_CONTEXT.set_metadata(serde_json::json!({
    "experiment": "A/B test",
    "version": "1.0.0"
}));
```


## Examples

Check out the [examples](examples/) directory for detailed usage:

- [`basic.rs`](examples/basic.rs) - Simple usage with minimal setup
- [`with_langfuse.rs`](examples/with_langfuse.rs) - Complete Langfuse integration with OpenTelemetry
- [`context.rs`](examples/context.rs) - Advanced usage with context attributes (session_id, user_id, tags)

To run the Langfuse example:

```bash
# Set your environment variables
export LANGFUSE_PUBLIC_KEY=pk-lf-...
export LANGFUSE_SECRET_KEY=sk-lf-...
export LANGFUSE_HOST=https://cloud.langfuse.com
export AZURE_OPENAI_ENDPOINT=...
export AZURE_OPENAI_API_KEY=...

# Run the example
cargo run --example with_langfuse
```

## How It Works

The middleware intercepts HTTP requests to OpenAI endpoints and:

1. Creates a root trace span if none exists (named "OpenAI-generation" by default)
2. Creates child spans for each API operation (chat, embeddings, etc.)
3. Extracts and records token usage, model information, and other metadata following [OpenTelemetry GenAI semantic conventions](https://opentelemetry.io/docs/specs/semconv/gen-ai/)
4. Applies any context attributes (session_id, user_id, tags) to the spans
5. Forwards the spans to your configured OpenTelemetry backend

**Note:** The `service.name` attribute should be set at the TracerProvider level (as shown in the Langfuse integration example), not at the span level. This follows OpenTelemetry best practices.

## OpenTelemetry GenAI Semantic Conventions

This library follows the [OpenTelemetry semantic conventions for GenAI](https://opentelemetry.io/docs/specs/semconv/gen-ai/) systems:

- **[Spans](https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-spans/)**: Records operation name, model, token usage, and system attributes
- **[Events](https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-events/)**: Captures message content and roles (when configured)
- **[Metrics](https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-metrics/)**: Tracks token usage and operation duration
- **[OpenAI-specific conventions](https://opentelemetry.io/docs/specs/semconv/gen-ai/openai/)**: Implements OpenAI-specific attributes like `gen_ai.system = "openai"`

Key attributes captured:
- `gen_ai.operation.name`: Operation type (chat, embeddings, etc.)
- `gen_ai.system`: Set to "openai" or "azure.ai.openai"
- `gen_ai.request.model`: Model name requested
- `gen_ai.usage.input_tokens`: Number of input tokens
- `gen_ai.usage.output_tokens`: Number of output tokens
- `gen_ai.response.model`: Actual model used for response

## Supported Operations

- ✅ Chat Completions (`/chat/completions`)
- ✅ Embeddings (`/embeddings`)
- ✅ Completions (`/completions`)
- ✅ Image Generation (`/images/generations`)
- ✅ Audio Transcription (`/audio/transcriptions`)
- ✅ Audio Translation (`/audio/translations`)

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- Built for use with [async-openai](https://github.com/64bit/async-openai)
- Integrates with [Langfuse](https://langfuse.com) for LLM observability
- Uses [OpenTelemetry](https://opentelemetry.io) for distributed tracing