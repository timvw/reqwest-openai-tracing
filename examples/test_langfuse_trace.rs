//! Minimal example to test Langfuse trace export without OpenAI dependencies

use opentelemetry::{global, trace::Tracer};
use opentelemetry_otlp::{SpanExporter, WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::trace::SdkTracerProvider;
use reqwest_openai_tracing::{
    build_langfuse_auth_header_from_env, build_langfuse_otlp_endpoint_from_env,
};
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Build endpoint and auth using the library functions
    let endpoint = build_langfuse_otlp_endpoint_from_env()?;
    let auth_header = build_langfuse_auth_header_from_env()?;

    // The endpoint needs /v1/traces appended for OTLP HTTP
    let otlp_endpoint = format!("{}/v1/traces", endpoint);

    println!("OTLP Endpoint: {}", otlp_endpoint);
    println!(
        "Auth header (first 30 chars): {}...",
        &auth_header[..30.min(auth_header.len())]
    );

    // Create headers
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), auth_header);

    // Create exporter
    let exporter = SpanExporter::builder()
        .with_http()
        .with_http_client(reqwest::Client::new())
        .with_endpoint(otlp_endpoint)
        .with_headers(headers)
        .build()?;

    // Create tracer provider
    let tracer_provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter)
        .with_resource(
            opentelemetry_sdk::Resource::builder()
                .with_attribute(opentelemetry::KeyValue::new(
                    "service.name",
                    "langfuse-trace-test",
                ))
                .build(),
        )
        .build();

    global::set_tracer_provider(tracer_provider.clone());

    // Create a test span
    let tracer = global::tracer("test-tracer");

    let session_id = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
    println!("\n📊 Creating test trace with session_id: {}", session_id);

    // Set Langfuse context
    use reqwest_openai_tracing::langfuse_context;
    langfuse_context::set_session_id(&session_id);
    langfuse_context::set_user_id("test-user-123");
    langfuse_context::add_tags(vec!["test".to_string(), "minimal".to_string()]);

    {
        let span = tracer
            .span_builder("test-operation")
            .with_attributes(vec![
                // Use Langfuse-specific attributes
                opentelemetry::KeyValue::new("langfuse.session.id", session_id.clone()),
                opentelemetry::KeyValue::new("langfuse.user.id", "test-user-123"),
                opentelemetry::KeyValue::new("langfuse.trace.name", "Test Trace"),
                opentelemetry::KeyValue::new(
                    "langfuse.trace.tags",
                    vec!["test", "minimal"].join(","),
                ),
                // Additional standard attributes
                opentelemetry::KeyValue::new("operation.type", "trace-test"),
                opentelemetry::KeyValue::new("test.timestamp", chrono::Local::now().to_rfc3339()),
            ])
            .start(&tracer);

        println!("📝 Span created, simulating work...");

        // Simulate some work
        sleep(Duration::from_millis(500)).await;

        // Create a nested span
        {
            let child_span = tracer
                .span_builder("nested-operation")
                .with_attributes(vec![
                    opentelemetry::KeyValue::new("langfuse.session.id", session_id.clone()),
                    opentelemetry::KeyValue::new("nested.level", 1i64),
                    opentelemetry::KeyValue::new("nested.type", "child"),
                ])
                .start(&tracer);

            sleep(Duration::from_millis(200)).await;
            drop(child_span);
        }

        println!("✅ Work completed, ending span");
        // End the span
        drop(span);
    }

    println!("\n🚀 Flushing spans to Langfuse...");
    if let Err(e) = tracer_provider.force_flush() {
        eprintln!("❌ Error flushing: {}", e);
    } else {
        println!("✅ Successfully flushed spans");
    }

    // Give time for export
    println!("⏳ Waiting for export to complete...");
    sleep(Duration::from_secs(2)).await;

    println!("🔌 Shutting down tracer provider...");
    tracer_provider.shutdown()?;

    println!("\n🎯 Done! Check Langfuse for traces:");
    println!("   Session ID: {}", session_id);
    println!("   Direct URL: https://cloud.langfuse.com/project/cmelic0jj00smad07ck19tdei/traces?sessionId={}", session_id);
    println!("\n📋 To verify, you can also search for:");
    println!("   - Service name: langfuse-trace-test");
    println!("   - User ID: test-user-123");
    println!("   - Tags: test, minimal");

    Ok(())
}

