//! Langfuse OpenTelemetry attribute management
//!
//! This module provides structured attribute management for Langfuse traces and observations,
//! following the patterns established by the Langfuse Python SDK.

use opentelemetry::KeyValue;
use serde_json::Value;

/// Langfuse-specific OpenTelemetry span attribute names
pub struct LangfuseAttributes;

#[allow(dead_code)]
impl LangfuseAttributes {
    // Trace attributes (following Python SDK naming)
    pub const TRACE_NAME: &'static str = "langfuse.trace.name";
    pub const TRACE_USER_ID: &'static str = "user.id";
    pub const TRACE_SESSION_ID: &'static str = "session.id";
    pub const TRACE_TAGS: &'static str = "langfuse.trace.tags";
    pub const TRACE_PUBLIC: &'static str = "langfuse.trace.public";
    pub const TRACE_METADATA: &'static str = "langfuse.trace.metadata";
    pub const TRACE_INPUT: &'static str = "langfuse.trace.input";
    pub const TRACE_OUTPUT: &'static str = "langfuse.trace.output";

    // Observation attributes
    pub const OBSERVATION_TYPE: &'static str = "langfuse.observation.type";
    pub const OBSERVATION_METADATA: &'static str = "langfuse.observation.metadata";
    pub const OBSERVATION_LEVEL: &'static str = "langfuse.observation.level";
    pub const OBSERVATION_STATUS_MESSAGE: &'static str = "langfuse.observation.status_message";
    pub const OBSERVATION_INPUT: &'static str = "langfuse.observation.input";
    pub const OBSERVATION_OUTPUT: &'static str = "langfuse.observation.output";

    // Generation-specific observation attributes
    pub const OBSERVATION_MODEL: &'static str = "langfuse.observation.model.name";
    pub const OBSERVATION_MODEL_PARAMETERS: &'static str = "langfuse.observation.model.parameters";
    pub const OBSERVATION_USAGE_TOTAL: &'static str = "langfuse.observation.usage.total";
    pub const OBSERVATION_USAGE_DETAILS: &'static str = "langfuse.observation.usage_details";
    pub const OBSERVATION_PROMPT_NAME: &'static str = "langfuse.observation.prompt.name";
    pub const OBSERVATION_PROMPT_VERSION: &'static str = "langfuse.observation.prompt.version";

    // General attributes
    pub const ENVIRONMENT: &'static str = "langfuse.environment";
    pub const RELEASE: &'static str = "langfuse.release";
    pub const VERSION: &'static str = "langfuse.version";
}

/// Builder for creating Langfuse trace attributes
pub struct TraceAttributesBuilder {
    attributes: Vec<KeyValue>,
}

#[allow(dead_code)]
impl TraceAttributesBuilder {
    pub fn new() -> Self {
        Self {
            attributes: Vec::new(),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.attributes
            .push(KeyValue::new(LangfuseAttributes::TRACE_NAME, name.into()));
        self
    }

    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.attributes.push(KeyValue::new(
            LangfuseAttributes::TRACE_USER_ID,
            user_id.into(),
        ));
        self
    }

    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.attributes.push(KeyValue::new(
            LangfuseAttributes::TRACE_SESSION_ID,
            session_id.into(),
        ));
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        // Convert to JSON array string for compatibility
        let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
        self.attributes
            .push(KeyValue::new(LangfuseAttributes::TRACE_TAGS, tags_json));
        self
    }

    pub fn with_metadata(mut self, metadata: Value) -> Self {
        let metadata_str = metadata.to_string();
        self.attributes.push(KeyValue::new(
            LangfuseAttributes::TRACE_METADATA,
            metadata_str,
        ));
        self
    }

    pub fn with_input(mut self, input: Value) -> Self {
        let input_str = input.to_string();
        self.attributes
            .push(KeyValue::new(LangfuseAttributes::TRACE_INPUT, input_str));
        self
    }

    pub fn with_output(mut self, output: Value) -> Self {
        let output_str = output.to_string();
        self.attributes
            .push(KeyValue::new(LangfuseAttributes::TRACE_OUTPUT, output_str));
        self
    }

    pub fn with_public(mut self, public: bool) -> Self {
        self.attributes
            .push(KeyValue::new(LangfuseAttributes::TRACE_PUBLIC, public));
        self
    }

    pub fn build(self) -> Vec<KeyValue> {
        self.attributes
    }
}

/// Builder for creating Langfuse observation/generation attributes
#[allow(dead_code)]
pub struct ObservationAttributesBuilder {
    attributes: Vec<KeyValue>,
}

#[allow(dead_code)]
impl ObservationAttributesBuilder {
    pub fn new(observation_type: &str) -> Self {
        let mut builder = Self {
            attributes: Vec::new(),
        };
        builder.attributes.push(KeyValue::new(
            LangfuseAttributes::OBSERVATION_TYPE,
            observation_type.to_string(),
        ));
        builder
    }

    pub fn generation() -> Self {
        Self::new("generation")
    }

    pub fn span() -> Self {
        Self::new("span")
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.attributes.push(KeyValue::new(
            LangfuseAttributes::OBSERVATION_MODEL,
            model.into(),
        ));
        self
    }

    pub fn with_metadata(mut self, metadata: Value) -> Self {
        let metadata_str = metadata.to_string();
        self.attributes.push(KeyValue::new(
            LangfuseAttributes::OBSERVATION_METADATA,
            metadata_str,
        ));
        self
    }

    pub fn with_input(mut self, input: Value) -> Self {
        let input_str = input.to_string();
        self.attributes.push(KeyValue::new(
            LangfuseAttributes::OBSERVATION_INPUT,
            input_str,
        ));
        self
    }

    pub fn with_output(mut self, output: Value) -> Self {
        let output_str = output.to_string();
        self.attributes.push(KeyValue::new(
            LangfuseAttributes::OBSERVATION_OUTPUT,
            output_str,
        ));
        self
    }

    pub fn with_usage_total(mut self, total: i64) -> Self {
        self.attributes.push(KeyValue::new(
            LangfuseAttributes::OBSERVATION_USAGE_TOTAL,
            total,
        ));
        self
    }

    pub fn with_prompt(mut self, name: impl Into<String>, version: Option<String>) -> Self {
        self.attributes.push(KeyValue::new(
            LangfuseAttributes::OBSERVATION_PROMPT_NAME,
            name.into(),
        ));
        if let Some(v) = version {
            self.attributes.push(KeyValue::new(
                LangfuseAttributes::OBSERVATION_PROMPT_VERSION,
                v,
            ));
        }
        self
    }

    pub fn build(self) -> Vec<KeyValue> {
        self.attributes
    }
}
