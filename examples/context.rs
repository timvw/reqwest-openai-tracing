//! Example showing how to use context attributes (session_id, user_id, tags)

use async_openai::{config::AzureConfig, Client};
use dotenv::dotenv;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_openai_tracing::{langfuse_context, HttpClientWithMiddleware, OpenAITracingMiddleware};
use std::error::Error;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("reqwest_openai_tracing=info".parse()?),
        )
        .init();

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

    // Example 1: Using global context helpers
    info!("Example 1: Setting context with global helpers");
    langfuse_context::set_session_id("session-123");
    langfuse_context::set_user_id("user-456");
    langfuse_context::add_tags(vec!["example".to_string(), "context".to_string()]);

    let request1 = async_openai::types::CreateChatCompletionRequestArgs::default()
        .messages(vec![
            async_openai::types::ChatCompletionRequestMessage::User(
                async_openai::types::ChatCompletionRequestUserMessage {
                    content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                        "Say hello".to_string(),
                    ),
                    name: None,
                },
            ),
        ])
        .max_tokens(10_u32)
        .build()?;

    let response1 = client.chat().create(request1).await?;
    info!("Response 1: {}", response1.choices[0].message.content.as_ref().unwrap_or(&String::new()));

    // Example 2: Using the context builder
    info!("Example 2: Setting context with builder pattern");
    langfuse_context::GLOBAL_CONTEXT.clear();
    
    let context = langfuse_context::LangfuseContextBuilder::new()
        .session_id("builder-session")
        .user_id("builder-user")
        .tags(vec!["built".to_string(), "with".to_string(), "builder".to_string()])
        .trace_name("Custom Trace Name")
        .build();
    
    // Apply the built context
    context.apply_to_current_span();

    let request2 = async_openai::types::CreateChatCompletionRequestArgs::default()
        .messages(vec![
            async_openai::types::ChatCompletionRequestMessage::User(
                async_openai::types::ChatCompletionRequestUserMessage {
                    content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                        "Say goodbye".to_string(),
                    ),
                    name: None,
                },
            ),
        ])
        .max_tokens(10_u32)
        .build()?;

    let response2 = client.chat().create(request2).await?;
    info!("Response 2: {}", response2.choices[0].message.content.as_ref().unwrap_or(&String::new()));

    // Example 3: Using custom metadata
    info!("Example 3: Setting context with metadata");
    langfuse_context::GLOBAL_CONTEXT.clear();
    langfuse_context::GLOBAL_CONTEXT
        .set_session_id("metadata-session")
        .set_user_id("metadata-user")
        .set_metadata(serde_json::json!({
            "experiment": "A/B test",
            "version": "1.0.0",
            "feature_flags": {
                "new_model": true,
                "streaming": false
            }
        }));

    let request3 = async_openai::types::CreateChatCompletionRequestArgs::default()
        .messages(vec![
            async_openai::types::ChatCompletionRequestMessage::User(
                async_openai::types::ChatCompletionRequestUserMessage {
                    content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                        "What is 2 + 2?".to_string(),
                    ),
                    name: None,
                },
            ),
        ])
        .max_tokens(10_u32)
        .build()?;

    let response3 = client.chat().create(request3).await?;
    info!("Response 3: {}", response3.choices[0].message.content.as_ref().unwrap_or(&String::new()));

    info!("All examples completed. Check your tracing backend for the different context attributes.");

    Ok(())
}