//! Example showing integration with Langfuse via OpenTelemetry

use async_openai::{config::AzureConfig, Client};
use dotenv::dotenv;
use opentelemetry::global;
use opentelemetry_otlp::{WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::trace::TracerProvider;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_openai_tracing::{
    build_langfuse_auth_header_from_env, build_langfuse_otlp_endpoint_from_env,
    HttpClientWithMiddleware, OpenAITracingMiddleware,
};
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

fn setup_tracing(service_name: &str) -> Result<(), Box<dyn Error>> {
    // Use the helper functions to build endpoint and headers from environment variables
    let endpoint = build_langfuse_otlp_endpoint_from_env()?;
    let auth_header = build_langfuse_auth_header_from_env()?;

    // Create headers HashMap for OpenTelemetry
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), auth_header);

    // Setup OpenTelemetry with OTLP exporter for Langfuse
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_endpoint(endpoint.clone())
        .with_headers(headers)
        .build()?;

    let tracer_provider = TracerProvider::builder()
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .with_resource(opentelemetry_sdk::Resource::new(vec![
            opentelemetry::KeyValue::new("service.name", service_name.to_string()),
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

    info!(
        "OpenTelemetry initialized with Langfuse backend at: {}",
        endpoint
    );
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables
    dotenv().ok();

    // Setup tracing with Langfuse
    // This expects the following environment variables:
    // - LANGFUSE_HOST: The base URL of your Langfuse instance (e.g., https://cloud.langfuse.com)
    // - LANGFUSE_PUBLIC_KEY: Your Langfuse public key
    // - LANGFUSE_SECRET_KEY: Your Langfuse secret key
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

    // Optional: Set context attributes for better organization in Langfuse
    use reqwest_openai_tracing::langfuse_context;
    langfuse_context::set_session_id("example-session-123");
    langfuse_context::set_user_id("example-user-456");
    langfuse_context::add_tags(vec!["example".to_string(), "langfuse".to_string()]);

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

    info!("Sending first request to OpenAI...");
    let response = client.chat().create(request).await?;

    let first_response = response.choices[0]
        .message
        .content
        .as_ref()
        .unwrap_or(&String::new())
        .clone();

    info!("First response: {}", first_response);

    if let Some(usage) = response.usage {
        info!(
            "Token usage - Prompt: {}, Completion: {}, Total: {}",
            usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
        );
    }

    // Build conversation history for follow-up request
    #[allow(deprecated)]
    let messages = vec![
        async_openai::types::ChatCompletionRequestMessage::User(
            async_openai::types::ChatCompletionRequestUserMessage {
                content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                    "What is the capital of France?".to_string(),
                ),
                name: None,
            },
        ),
        async_openai::types::ChatCompletionRequestMessage::Assistant(
            async_openai::types::ChatCompletionRequestAssistantMessage {
                content: Some(
                    async_openai::types::ChatCompletionRequestAssistantMessageContent::Text(
                        first_response,
                    ),
                ),
                name: None,
                tool_calls: None,
                audio: None,
                refusal: None,
                function_call: None,
            },
        ),
        async_openai::types::ChatCompletionRequestMessage::User(
            async_openai::types::ChatCompletionRequestUserMessage {
                content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                    "Please repeat your response, but in French language.".to_string(),
                ),
                name: None,
            },
        ),
    ];

    // Make follow-up request
    let follow_up_request = async_openai::types::CreateChatCompletionRequestArgs::default()
        .messages(messages)
        .temperature(0.7)
        .max_tokens(50_u32)
        .build()?;

    info!("Sending follow-up request to OpenAI...");
    let follow_up_response = client.chat().create(follow_up_request).await?;

    info!(
        "Follow-up response (in French): {}",
        follow_up_response.choices[0]
            .message
            .content
            .as_ref()
            .unwrap_or(&String::new())
    );

    if let Some(usage) = follow_up_response.usage {
        info!(
            "Follow-up token usage - Prompt: {}, Completion: {}, Total: {}",
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
