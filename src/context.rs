//! Langfuse context helpers for setting trace attributes
//! Similar to the Python SDK's langfuse_context

#![allow(dead_code)]

use crate::attributes::LangfuseAttributes;
use opentelemetry::KeyValue;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Thread-safe storage for Langfuse context attributes
#[derive(Clone)]
pub struct LangfuseContext {
    attributes: Arc<RwLock<HashMap<String, String>>>,
}

impl LangfuseContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self {
            attributes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set the session ID for the current trace
    pub fn set_session_id(&self, session_id: impl Into<String>) -> &Self {
        self.set_attribute(LangfuseAttributes::TRACE_SESSION_ID, session_id);
        self
    }

    /// Set the user ID for the current trace
    pub fn set_user_id(&self, user_id: impl Into<String>) -> &Self {
        self.set_attribute(LangfuseAttributes::TRACE_USER_ID, user_id);
        self
    }

    /// Add tags to the current trace
    pub fn add_tags(&self, tags: Vec<String>) -> &Self {
        // Convert to JSON array string for compatibility
        let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
        self.set_attribute(LangfuseAttributes::TRACE_TAGS, tags_json);
        self
    }

    /// Add a single tag
    pub fn add_tag(&self, tag: impl Into<String>) -> &Self {
        let tag = tag.into();
        let mut attrs = self.attributes.write().unwrap();

        // Append to existing tags if present
        if let Some(existing) = attrs.get(LangfuseAttributes::TRACE_TAGS) {
            // Parse existing JSON array, add tag, and re-serialize
            if let Ok(mut tags_vec) = serde_json::from_str::<Vec<String>>(existing) {
                tags_vec.push(tag);
                let tags_json =
                    serde_json::to_string(&tags_vec).unwrap_or_else(|_| "[]".to_string());
                attrs.insert(LangfuseAttributes::TRACE_TAGS.to_string(), tags_json);
            } else {
                // Fallback if existing isn't valid JSON
                attrs.insert(
                    LangfuseAttributes::TRACE_TAGS.to_string(),
                    format!("[\"{}\"]", tag),
                );
            }
        } else {
            attrs.insert(
                LangfuseAttributes::TRACE_TAGS.to_string(),
                format!("[\"{}\"]", tag),
            );
        }
        drop(attrs);
        self
    }

    /// Set metadata as JSON string
    pub fn set_metadata(&self, metadata: serde_json::Value) -> &Self {
        let metadata_str = metadata.to_string();
        self.set_attribute(LangfuseAttributes::TRACE_METADATA, metadata_str);
        self
    }

    /// Set a custom attribute
    pub fn set_attribute(&self, key: impl Into<String>, value: impl Into<String>) -> &Self {
        let mut attrs = self.attributes.write().unwrap();
        attrs.insert(key.into(), value.into());
        drop(attrs);
        self
    }

    /// Set the trace name
    pub fn set_trace_name(&self, name: impl Into<String>) -> &Self {
        self.set_attribute(LangfuseAttributes::TRACE_NAME, name);
        self
    }

    /// Set the trace ID (useful for linking traces)
    pub fn set_trace_id(&self, trace_id: impl Into<String>) -> &Self {
        self.set_attribute("langfuse.trace.id", trace_id);
        self
    }

    /// Set the parent trace ID for nested traces
    pub fn set_parent_trace_id(&self, parent_id: impl Into<String>) -> &Self {
        self.set_attribute("langfuse.parent.trace.id", parent_id);
        self
    }

    /// Clear all attributes
    pub fn clear(&self) {
        let mut attrs = self.attributes.write().unwrap();
        attrs.clear();
    }

    /// Note: Attributes are automatically applied when creating new spans in the middleware.
    /// This method exists for API compatibility with Python SDK but doesn't need to do anything
    /// since we apply attributes at span creation time.
    pub fn apply_to_current_span(&self) {
        // Attributes are automatically applied in the middleware when creating spans
        // This is a no-op for API compatibility
    }

    /// Get all current attributes as key-value pairs
    pub fn get_attributes(&self) -> Vec<KeyValue> {
        let attrs = self.attributes.read().unwrap();
        attrs
            .iter()
            .map(|(k, v)| KeyValue::new(k.clone(), v.clone()))
            .collect()
    }

    /// Check if a specific attribute is set
    pub fn has_attribute(&self, key: &str) -> bool {
        let attrs = self.attributes.read().unwrap();
        attrs.contains_key(key)
    }

    /// Get a specific attribute value
    pub fn get_attribute(&self, key: &str) -> Option<String> {
        let attrs = self.attributes.read().unwrap();
        attrs.get(key).cloned()
    }
}

impl Default for LangfuseContext {
    fn default() -> Self {
        Self::new()
    }
}

// Global context instance (optional - users can create their own)
lazy_static::lazy_static! {
    pub static ref GLOBAL_CONTEXT: LangfuseContext = LangfuseContext::new();
}

/// Helper function to set session ID on global context
pub fn set_session_id(session_id: impl Into<String>) {
    GLOBAL_CONTEXT.set_session_id(session_id);
}

/// Helper function to set user ID on global context
pub fn set_user_id(user_id: impl Into<String>) {
    GLOBAL_CONTEXT.set_user_id(user_id);
}

/// Helper function to add tags on global context
pub fn add_tags(tags: Vec<String>) {
    GLOBAL_CONTEXT.add_tags(tags);
}

/// Helper function to apply global context to current span
pub fn apply_context() {
    GLOBAL_CONTEXT.apply_to_current_span();
}

/// Builder pattern for fluent API
pub struct LangfuseContextBuilder {
    context: LangfuseContext,
}

impl LangfuseContextBuilder {
    pub fn new() -> Self {
        Self {
            context: LangfuseContext::new(),
        }
    }

    pub fn session_id(self, session_id: impl Into<String>) -> Self {
        self.context.set_session_id(session_id);
        self
    }

    pub fn user_id(self, user_id: impl Into<String>) -> Self {
        self.context.set_user_id(user_id);
        self
    }

    pub fn tags(self, tags: Vec<String>) -> Self {
        self.context.add_tags(tags);
        self
    }

    pub fn metadata(self, metadata: serde_json::Value) -> Self {
        self.context.set_metadata(metadata);
        self
    }

    pub fn trace_name(self, name: impl Into<String>) -> Self {
        self.context.set_trace_name(name);
        self
    }

    pub fn build(self) -> LangfuseContext {
        self.context
    }

    pub fn apply(self) {
        self.context.apply_to_current_span();
    }
}
