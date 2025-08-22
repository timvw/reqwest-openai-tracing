use crate::attributes::TraceAttributesBuilder;
use http::Extensions;
use opentelemetry::trace::{FutureExt, Span, SpanKind, Status, TraceContextExt, Tracer};
use opentelemetry::{global, Context, KeyValue};
use opentelemetry_semantic_conventions::attribute::{
    ERROR_TYPE, GEN_AI_OPERATION_NAME, GEN_AI_REQUEST_MODEL, GEN_AI_SYSTEM,
    HTTP_RESPONSE_STATUS_CODE,
};
use opentelemetry_semantic_conventions::attribute::{
    GEN_AI_USAGE_INPUT_TOKENS, GEN_AI_USAGE_OUTPUT_TOKENS,
};
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next, Result};
use serde_json::{json, Value};
use std::time::Instant;

/// Middleware that automatically creates OpenTelemetry spans for OpenAI API calls
#[allow(dead_code)]
pub struct OpenAITracingMiddleware;

impl Default for OpenAITracingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenAITracingMiddleware {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    fn extract_operation_from_path(path: &str) -> (&str, &str) {
        if path.contains("/chat/completions") {
            ("chat", "chat.completions")
        } else if path.contains("/completions") {
            ("completion", "completions")
        } else if path.contains("/embeddings") {
            ("embedding", "embeddings")
        } else if path.contains("/images/generations") {
            ("image", "images.generations")
        } else {
            ("unknown", "unknown")
        }
    }
}

#[async_trait::async_trait]
impl Middleware for OpenAITracingMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        let tracer = global::tracer("openai-middleware");
        let start_time = Instant::now();

        // Extract request information
        let path = req.url().path().to_string();
        let (operation_type, operation_name) = Self::extract_operation_from_path(&path);

        // Note: Following Python SDK pattern - root traces created by middleware
        // don't automatically get input/output from child observations

        // Get the current context to check if we have a parent trace
        let current_context = Context::current();
        let parent_span = current_context.span();

        // Check if we need to create a root trace and handle it
        if !parent_span.span_context().is_valid() {
            // No active span - create a root trace for Langfuse
            // Check if trace name is set in context, otherwise use Python SDK default
            let trace_name = crate::context::GLOBAL_CONTEXT
                .get_attribute(crate::attributes::LangfuseAttributes::TRACE_NAME)
                .unwrap_or_else(|| "OpenAI-generation".to_string());

            // Build attributes using the builder pattern
            let builder = TraceAttributesBuilder::new().with_name(trace_name.clone());
            let mut root_attributes = builder.build();

            // Apply any programmatically-set context attributes to the root span
            let context_attrs = crate::context::GLOBAL_CONTEXT.get_attributes();
            root_attributes.extend(context_attrs);

            let root_span = tracer
                .span_builder(trace_name)
                .with_kind(SpanKind::Internal)
                .with_attributes(root_attributes)
                .start(&tracer);

            // Make it the current context
            let cx = Context::current_with_span(root_span);

            // Process the request in the new span context using with_context
            let result = self
                .process_request_with_attributes(
                    req,
                    extensions,
                    next,
                    operation_type,
                    operation_name,
                    &path,
                    start_time,
                )
                .with_context(cx.clone())
                .await;

            // End the root span
            cx.span().end();

            result
        } else {
            // We have a parent span, use it
            self.process_request_with_attributes(
                req,
                extensions,
                next,
                operation_type,
                operation_name,
                &path,
                start_time,
            )
            .await
        }
    }
}

impl OpenAITracingMiddleware {
    #[allow(clippy::too_many_arguments)]
    async fn process_request_with_attributes(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
        operation_type: &str,
        operation_name: &str,
        path: &str,
        start_time: Instant,
    ) -> Result<Response> {
        let tracer = global::tracer("openai-middleware");

        // Try to extract and parse the request body to get the actual input
        let mut model: Option<String> = None;
        let mut observation_input: Option<Value> = None;

        // Try to extract deployment/model from URL for Azure
        // Azure URL format: .../openai/deployments/{deployment-id}/chat/completions
        if path.contains("/deployments/") {
            if let Some(start) = path.find("/deployments/") {
                let after_deployments = &path[start + "/deployments/".len()..];
                if let Some(end) = after_deployments.find('/') {
                    model = Some(after_deployments[..end].to_string());
                }
            }
        }

        // Parse request body based on operation type
        if let Some(body) = req.body() {
            if let Some(bytes) = body.as_bytes() {
                if let Ok(json) = serde_json::from_slice::<Value>(bytes) {
                    // Extract model from request body (for OpenAI, not Azure)
                    // Only override if model field exists and is not empty
                    if let Some(m) = json.get("model") {
                        if let Some(model_str) = m.as_str() {
                            if !model_str.is_empty() {
                                model = Some(model_str.to_string());
                            }
                        }
                    }

                    // Store the input for the observation based on operation type
                    observation_input = match operation_type {
                        "chat" => {
                            // Chat completions: extract messages
                            json.get("messages").map(|messages| {
                                json!({
                                    "messages": messages,
                                })
                            })
                        }
                        "completion" => {
                            // Text completions: extract prompt
                            json.get("prompt").map(|prompt| {
                                json!({
                                    "prompt": prompt,
                                })
                            })
                        }
                        "embedding" => {
                            // Embeddings: extract input
                            json.get("input").map(|input| {
                                json!({
                                    "input": input,
                                })
                            })
                        }
                        "image" => {
                            // Image generation: extract prompt and parameters
                            let mut image_input = serde_json::Map::new();
                            if let Some(prompt) = json.get("prompt") {
                                image_input.insert("prompt".to_string(), prompt.clone());
                            }
                            if let Some(n) = json.get("n") {
                                image_input.insert("n".to_string(), n.clone());
                            }
                            if let Some(size) = json.get("size") {
                                image_input.insert("size".to_string(), size.clone());
                            }
                            if !image_input.is_empty() {
                                Some(Value::Object(image_input))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };
                }
            }
        }

        // Create span with OpenAI-specific attributes following Langfuse Python SDK patterns
        let mut attributes = vec![
            // OpenAI/LLM specific attributes (using semantic conventions)
            KeyValue::new(GEN_AI_SYSTEM, "openai"),
            KeyValue::new(GEN_AI_OPERATION_NAME, operation_type.to_string()),
            // Langfuse observation attributes (matching Python SDK)
            KeyValue::new("langfuse.observation.type", "generation"),
        ];

        // Only add model attributes if we could determine the model
        if let Some(ref model_name) = model {
            // Use semantic convention for model
            attributes.push(KeyValue::new(GEN_AI_REQUEST_MODEL, model_name.clone()));
            // Also add Langfuse-specific model attribute (matching Python SDK)
            attributes.push(KeyValue::new(
                "langfuse.observation.model.name",
                model_name.clone(),
            ));
        }

        // Add observation input if available
        if let Some(ref input) = observation_input {
            attributes.push(KeyValue::new(
                "langfuse.observation.input",
                input.to_string(),
            ));
        }

        // Apply any attributes from the global LangfuseContext (matching Python SDK behavior)
        // Note: These must be set programmatically via langfuse_context functions
        // This matches the Python SDK which requires calling langfuse_context.update_current_trace()
        let context_attrs = crate::context::GLOBAL_CONTEXT.get_attributes();
        attributes.extend(context_attrs);

        let mut span = tracer
            .span_builder(format!("OpenAI {}", operation_name))
            .with_kind(SpanKind::Client)
            .with_attributes(attributes)
            .start(&tracer);

        // Execute the request
        let response = next.run(req, extensions).await;

        // Record response information
        let response = match response {
            Ok(res) => {
                let status = res.status();
                span.set_attribute(KeyValue::new(
                    HTTP_RESPONSE_STATUS_CODE,
                    status.as_u16() as i64,
                ));

                if status.is_success() {
                    span.set_status(Status::Ok);

                    // Try to parse response body to set output and token usage
                    // Buffer the response body to parse it
                    match res.bytes().await {
                        Ok(bytes) => {
                            // Parse the response
                            if let Ok(response_json) = serde_json::from_slice::<Value>(&bytes) {
                                // Extract and set output based on operation type
                                let observation_output = match operation_type {
                                    "chat" => {
                                        // Chat completions: extract message from first choice
                                        response_json
                                            .get("choices")
                                            .and_then(|choices| choices.as_array())
                                            .and_then(|arr| arr.first())
                                            .and_then(|choice| choice.get("message"))
                                            .map(|message| {
                                                json!({
                                                    "choices": [{
                                                        "message": message
                                                    }]
                                                })
                                            })
                                    }
                                    "completion" => {
                                        // Text completions: extract text from choices
                                        response_json
                                            .get("choices")
                                            .and_then(|choices| choices.as_array())
                                            .map(|choices_arr| {
                                                let texts: Vec<_> = choices_arr
                                                    .iter()
                                                    .filter_map(|c| c.get("text"))
                                                    .collect();
                                                json!({
                                                    "choices": texts
                                                })
                                            })
                                    }
                                    "embedding" => {
                                        // Embeddings: extract embedding vectors
                                        response_json
                                            .get("data")
                                            .and_then(|data| data.as_array())
                                            .map(|data_arr| {
                                                json!({
                                                    "embeddings_count": data_arr.len(),
                                                    // Don't include full vectors as they're too large
                                                    "model": response_json.get("model")
                                                })
                                            })
                                    }
                                    "image" => {
                                        // Image generation: extract URLs or b64_json
                                        response_json
                                            .get("data")
                                            .and_then(|data| data.as_array())
                                            .map(|data_arr| {
                                                let urls: Vec<_> = data_arr
                                                    .iter()
                                                    .filter_map(|item| item.get("url"))
                                                    .collect();
                                                let b64_images_count = data_arr
                                                    .iter()
                                                    .filter(|item| item.get("b64_json").is_some())
                                                    .count();
                                                json!({
                                                    "urls": urls,
                                                    "b64_images_count": b64_images_count
                                                })
                                            })
                                    }
                                    _ => None,
                                };

                                // Set observation output if available
                                if let Some(output) = observation_output {
                                    span.set_attribute(KeyValue::new(
                                        "langfuse.observation.output",
                                        output.to_string(),
                                    ));
                                }

                                // Set token usage on span (if available)
                                if let Some(usage) = response_json.get("usage") {
                                    if let Some(prompt_tokens) =
                                        usage.get("prompt_tokens").and_then(|v| v.as_i64())
                                    {
                                        span.set_attribute(KeyValue::new(
                                            GEN_AI_USAGE_INPUT_TOKENS,
                                            prompt_tokens,
                                        ));
                                    }
                                    if let Some(completion_tokens) =
                                        usage.get("completion_tokens").and_then(|v| v.as_i64())
                                    {
                                        span.set_attribute(KeyValue::new(
                                            GEN_AI_USAGE_OUTPUT_TOKENS,
                                            completion_tokens,
                                        ));
                                    }
                                    // Total tokens is not in semantic conventions, but useful for Langfuse
                                    if let Some(total_tokens) =
                                        usage.get("total_tokens").and_then(|v| v.as_i64())
                                    {
                                        span.set_attribute(KeyValue::new(
                                            "langfuse.observation.usage.total",
                                            total_tokens,
                                        ));
                                    }
                                }
                            }

                            // Reconstruct the response with the buffered body
                            let new_response = Response::from(
                                http::Response::builder()
                                    .status(status)
                                    .body(bytes)
                                    .unwrap(),
                            );
                            Ok(new_response)
                        }
                        Err(e) => {
                            span.set_status(Status::error(format!(
                                "Failed to read response body: {}",
                                e
                            )));
                            span.set_attribute(KeyValue::new(ERROR_TYPE, e.to_string()));
                            Err(reqwest_middleware::Error::Reqwest(e))
                        }
                    }
                } else {
                    span.set_status(Status::error(format!("HTTP {}", status)));
                    Ok(res)
                }
            }
            Err(e) => {
                span.set_status(Status::error(format!("Request failed: {}", e)));
                span.set_attribute(KeyValue::new(ERROR_TYPE, e.to_string()));
                Err(e)
            }
        };

        // Record duration
        let duration_ms = start_time.elapsed().as_millis() as i64;
        span.set_attribute(KeyValue::new("duration_ms", duration_ms));

        span.end();

        response
    }
}
