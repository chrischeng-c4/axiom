use crate::error::{NovaError, NovaResult};
use crate::types::{Message, TokenUsage, ToolCall};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// LLM completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub messages: Vec<Message>,
    pub model: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,

    #[serde(default)]
    pub stream: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,

    /// JSON Schema for structured output. Provider-specific handling:
    /// - OpenAI: response_format with json_schema
    /// - Claude: tool-use with schema as single tool
    /// - Gemini: response_mime_type + response_schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_schema: Option<serde_json::Value>,

    #[serde(default)]
    pub extras: HashMap<String, serde_json::Value>,
}

/// Tool definition for LLM function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// LLM completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub content: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,

    pub finish_reason: String,
    pub usage: TokenUsage,
    pub model: String,

    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Streaming chunk from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub content: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,

    pub is_final: bool,
}

/// Unified streaming response wrapper
pub type StreamResponse =
    std::pin::Pin<Box<dyn futures::Stream<Item = NovaResult<StreamChunk>> + Send>>;

/// Unified LLM provider trait
#[async_trait]
pub trait LLMProvider: Send + Sync {
    fn provider_name(&self) -> &str;
    fn supported_models(&self) -> Vec<String>;
    async fn complete(&self, request: CompletionRequest) -> NovaResult<CompletionResponse>;
    async fn complete_stream(&self, request: CompletionRequest) -> NovaResult<StreamResponse>;

    fn validate_model(&self, model: &str) -> NovaResult<()> {
        if self.supported_models().contains(&model.to_string()) {
            Ok(())
        } else {
            Err(NovaError::ModelNotFound(format!(
                "Model '{}' is not supported by provider '{}'",
                model,
                self.provider_name()
            )))
        }
    }
}

impl CompletionRequest {
    pub fn new(messages: Vec<Message>, model: impl Into<String>) -> Self {
        Self {
            messages,
            model: model.into(),
            temperature: None,
            max_tokens: None,
            top_p: None,
            stop: None,
            stream: false,
            tools: None,
            response_schema: None,
            extras: HashMap::new(),
        }
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    pub fn with_response_schema(mut self, schema: serde_json::Value) -> Self {
        self.response_schema = Some(schema);
        self
    }
}
