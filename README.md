# reqwest-openai-tracing

OpenTelemetry tracing middleware for OpenAI API calls made with reqwest.

[![Crates.io](https://img.shields.io/crates/v/reqwest-openai-tracing.svg)](https://crates.io/crates/reqwest-openai-tracing)
[![Documentation](https://docs.rs/reqwest-openai-tracing/badge.svg)](https://docs.rs/reqwest-openai-tracing)
[![License](https://img.shields.io/crates/l/reqwest-openai-tracing.svg)](https://github.com/timvw/reqwest-openai-tracing#license)

## Features

- 🔍 **Automatic Tracing**: Automatically creates OpenTelemetry spans for OpenAI API calls
- 📊 **Token Usage Tracking**: Records prompt and completion token usage in span attributes
- 🏷️ **Langfuse Integration**: Seamlessly integrates with Langfuse via OpenTelemetry
- 🎯 **Context Attributes**: Set session IDs, user IDs, tags, and metadata for traces
- 🚀 **async-openai Compatible**: Works with the async-openai library via HttpClient trait
- 🔧 **Flexible**: Works with any OpenTelemetry backend (Jaeger, Zipkin, Langfuse, etc.)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
reqwest-openai-tracing = "0.1.0"
async-openai = { git = "https://github.com/timvw/async-openai.git", rev = "baadc6a" }
reqwest-middleware = "0.4"
```

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

## Setting Context Attributes

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

## Langfuse Integration

To send traces to Langfuse, configure OpenTelemetry with the OTLP exporter:

```rust
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::TracerProvider;

let tracer_provider = TracerProvider::builder()
    .with_batch_exporter(
        opentelemetry_otlp::new_exporter()
            .http()
            .with_endpoint("https://cloud.langfuse.com/api/public/otel")
            .with_headers(/* your auth headers */)
            .build()?,
        opentelemetry_sdk::runtime::Tokio,
    )
    .build();
```

## Environment Variables

When using with Langfuse, set these environment variables:

```bash
# Langfuse configuration
LANGFUSE_SECRET_KEY=sk-lf-...
LANGFUSE_PUBLIC_KEY=pk-lf-...
LANGFUSE_HOST=https://cloud.langfuse.com

# OpenTelemetry configuration for Langfuse
OTEL_EXPORTER_OTLP_ENDPOINT=https://cloud.langfuse.com/api/public/otel
OTEL_EXPORTER_OTLP_HEADERS=Authorization=Basic <base64_encoded_keys>
```

## Examples

Check out the [examples](examples/) directory for more detailed usage:

- [`basic.rs`](examples/basic.rs) - Simple usage without OpenTelemetry setup
- [`with_langfuse.rs`](examples/with_langfuse.rs) - Complete Langfuse integration
- [`context.rs`](examples/context.rs) - Using context attributes

## How It Works

The middleware intercepts HTTP requests to OpenAI endpoints and:

1. Creates a root trace span if none exists (named "OpenAI-generation" by default)
2. Creates child spans for each API operation (chat, embeddings, etc.)
3. Extracts and records token usage, model information, and other metadata
4. Applies any context attributes (session_id, user_id, tags) to the spans
5. Forwards the spans to your configured OpenTelemetry backend

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