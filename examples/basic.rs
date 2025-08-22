//! Basic example showing how to use reqwest-openai-tracing middleware

use async_openai::{config::AzureConfig, Client};
use dotenv::dotenv;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_openai_tracing::{HttpClientWithMiddleware, OpenAITracingMiddleware};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables
    dotenv().ok();

    // Initialize tracing (you would normally set up OpenTelemetry here)
    tracing_subscriber::fmt::init();

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

    // Make a request - it will be automatically traced
    let request = async_openai::types::CreateChatCompletionRequestArgs::default()
        .messages(vec![
            async_openai::types::ChatCompletionRequestMessage::User(
                async_openai::types::ChatCompletionRequestUserMessage {
                    content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                        "Say hello in one word".to_string(),
                    ),
                    name: None,
                },
            ),
        ])
        .temperature(0.7)
        .max_tokens(10_u32)
        .build()?;

    let response = client.chat().create(request).await?;
    
    println!(
        "Response: {}",
        response.choices[0]
            .message
            .content
            .as_ref()
            .unwrap_or(&String::new())
    );

    if let Some(usage) = response.usage {
        println!(
            "Token usage - Prompt: {}, Completion: {}, Total: {}",
            usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
        );
    }

    Ok(())
}