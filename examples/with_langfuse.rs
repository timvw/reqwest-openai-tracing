//! Example showing integration with Langfuse via OpenTelemetry

use async_openai::{config::AzureConfig, Client};
use dotenv::dotenv;
use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::{self, TracerProvider};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_openai_tracing::{HttpClientWithMiddleware, OpenAITracingMiddleware};
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

fn setup_tracing(service_name: &str) -> Result<(), Box<dyn Error>> {
    // Setup OpenTelemetry with OTLP exporter for Langfuse
    let tracer_provider = TracerProvider::builder()
        .with_batch_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint(std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")?)
                .with_headers(
                    std::env::var("OTEL_EXPORTER_OTLP_HEADERS")?
                        .split(',')
                        .filter_map(|h| {
                            let parts: Vec<&str> = h.splitn(2, '=').collect();
                            if parts.len() == 2 {
                                Some((parts[0].to_string(), parts[1].to_string()))
                            } else {
                                None
                            }
                        })
                        .collect(),
                )
                .build()?,
            opentelemetry_sdk::runtime::Tokio,
        )
        .with_resource(opentelemetry_sdk::Resource::new(vec![
            opentelemetry::KeyValue::new("service.name", service_name),
        ]))
        .build();

    global::set_tracer_provider(tracer_provider);

    // Setup tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("reqwest_openai_tracing=info".parse()?),
        )
        .init();

    info!("OpenTelemetry initialized with Langfuse backend");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables
    dotenv().ok();

    // Setup tracing with Langfuse
    setup_tracing("reqwest-openai-example")?;

    // Create reqwest client with tracing middleware
    let reqwest_client = reqwest::Client::new();
    let middleware_client: ClientWithMiddleware = ClientBuilder::new(reqwest_client)
        .with(OpenAITracingMiddleware::new())
        .build();

    // Wrap it to implement HttpClient trait
    let http_client = HttpClientWithMiddleware::new(middleware_client);

    // Setup Azure OpenAI config
    let config = AzureConfig::new()
        .with_api_base(std::env::var("AZURE_OPENAI_ENDPOINT")?)
        .with_api_key(std::env::var("AZURE_OPENAI_API_KEY")?)
        .with_deployment_id(
            std::env::var("AZURE_OPENAI_DEPLOYMENT").unwrap_or_else(|_| "gpt-4".to_string()),
        )
        .with_api_version("2024-02-01");

    // Create client with our middleware
    let client = Client::build(http_client, config, Default::default());

    // Make a request - it will be traced and sent to Langfuse
    let request = async_openai::types::CreateChatCompletionRequestArgs::default()
        .messages(vec![
            async_openai::types::ChatCompletionRequestMessage::User(
                async_openai::types::ChatCompletionRequestUserMessage {
                    content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                        "What is the capital of France?".to_string(),
                    ),
                    name: None,
                },
            ),
        ])
        .temperature(0.7)
        .max_tokens(50_u32)
        .build()?;

    info!("Sending request to OpenAI...");
    let response = client.chat().create(request).await?;
    
    info!(
        "Response: {}",
        response.choices[0]
            .message
            .content
            .as_ref()
            .unwrap_or(&String::new())
    );

    if let Some(usage) = response.usage {
        info!(
            "Token usage - Prompt: {}, Completion: {}, Total: {}",
            usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
        );
    }

    info!("Exporting traces to Langfuse...");
    
    // Give time for traces to flush
    sleep(Duration::from_secs(2)).await;
    
    // Shutdown tracing
    global::shutdown_tracer_provider();
    sleep(Duration::from_secs(1)).await;

    info!("Traces have been sent to Langfuse. Check your dashboard for details.");

    Ok(())
}