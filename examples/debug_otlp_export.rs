//! Debug OTLP export to understand what's being sent

use base64::Engine;
use opentelemetry::{global, trace::Tracer};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables
    dotenv::dotenv().ok();

    // First, let's test the endpoint directly with curl-like request
    let langfuse_host = std::env::var("LANGFUSE_HOST")?;
    let public_key = std::env::var("LANGFUSE_PUBLIC_KEY")?;
    let secret_key = std::env::var("LANGFUSE_SECRET_KEY")?;

    let endpoint = format!(
        "{}/api/public/otel/v1/traces",
        langfuse_host.trim_end_matches('/')
    );
    let auth_string = format!("{}:{}", public_key, secret_key);
    let encoded = base64::engine::general_purpose::STANDARD.encode(auth_string.as_bytes());
    let auth_header = format!("Basic {}", encoded);

    println!("Testing endpoint: {}", endpoint);
    println!(
        "Auth header (first 50 chars): {}...",
        &auth_header[..50.min(auth_header.len())]
    );

    // Create a minimal OTLP trace export request
    // We'll use the protobuf format as expected by Langfuse
    let client = reqwest::Client::new();

    // First test: Empty request to check endpoint
    println!("\n1. Testing endpoint with empty request...");
    let response = client
        .post(&endpoint)
        .header("Authorization", &auth_header)
        .header("Content-Type", "application/x-protobuf")
        .body(vec![])
        .send()
        .await?;

    println!("   Status: {}", response.status());
    println!("   Headers: {:?}", response.headers());
    let body = response.text().await?;
    println!("   Body: {}", body);

    // Now let's try with the actual OpenTelemetry SDK
    println!("\n2. Setting up OpenTelemetry with detailed logging...");

    use opentelemetry_otlp::{SpanExporter, WithExportConfig, WithHttpConfig};
    use opentelemetry_sdk::trace::SdkTracerProvider;
    use std::collections::HashMap;

    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), auth_header.clone());

    // Try with different configurations
    println!("\n3. Creating exporter with endpoint: {}", endpoint);

    let exporter = SpanExporter::builder()
        .with_http()
        .with_http_client(reqwest::Client::new())
        .with_endpoint(&endpoint)
        .with_headers(headers)
        .with_timeout(std::time::Duration::from_secs(10))
        .build()?;

    let tracer_provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter)
        .with_resource(
            opentelemetry_sdk::Resource::builder()
                .with_attribute(opentelemetry::KeyValue::new("service.name", "debug-test"))
                .with_attribute(opentelemetry::KeyValue::new("service.version", "1.0.0"))
                .build(),
        )
        .build();

    global::set_tracer_provider(tracer_provider.clone());

    // Create a simple span
    let tracer = global::tracer("debug-tracer");
    let timestamp = chrono::Local::now();
    let session_id = timestamp.format("%Y%m%d%H%M%S").to_string();

    println!("\n4. Creating span with session_id: {}", session_id);
    println!("   Timestamp: {}", timestamp.to_rfc3339());

    {
        let mut span = tracer
            .span_builder("debug-operation")
            .with_start_time(std::time::SystemTime::now())
            .with_attributes(vec![
                opentelemetry::KeyValue::new("session.id", session_id.clone()),
                opentelemetry::KeyValue::new("debug.test", true),
                opentelemetry::KeyValue::new("timestamp", timestamp.to_rfc3339()),
            ])
            .start(&tracer);

        // Add an event to the span
        use opentelemetry::trace::Span;
        span.add_event(
            "test-event",
            vec![
                opentelemetry::KeyValue::new("event.type", "debug"),
                opentelemetry::KeyValue::new("event.message", "This is a test event"),
            ],
        );

        // Simulate some work
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Explicitly end the span
        span.end();
    }

    println!("\n5. Forcing flush...");
    match tracer_provider.force_flush() {
        Ok(_) => println!("   ✅ Flush successful"),
        Err(e) => println!("   ❌ Flush error: {:?}", e),
    }

    // Wait a bit for export
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    println!("\n6. Shutting down...");
    match tracer_provider.shutdown() {
        Ok(_) => println!("   ✅ Shutdown successful"),
        Err(e) => println!("   ❌ Shutdown error: {:?}", e),
    }

    println!("\n📊 Summary:");
    println!("   Endpoint: {}", endpoint);
    println!("   Session ID: {}", session_id);
    println!("   Timestamp: {}", timestamp);
    println!("\n🔗 Check URL: https://cloud.langfuse.com/project/cmelic0jj00smad07ck19tdei/traces?sessionId={}", session_id);

    Ok(())
}

