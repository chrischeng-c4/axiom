use super::provider::{
    CompletionRequest, CompletionResponse, LLMProvider, StreamResponse, ToolDefinition,
};
use crate::error::{NovaError, NovaResult};
use crate::types::{Message, Role, TokenUsage, ToolCall};
use async_trait::async_trait;
use mambalibs_http::client::{HttpClient, HttpClientConfig, HttpMethod, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;

/// OpenAI LLM provider
pub struct OpenAIProvider {
    client: Arc<HttpClient>,
    api_key: String,
    default_model: String,
}

impl OpenAIProvider {
    pub fn new(api_key: impl Into<String>) -> NovaResult<Self> {
        Self::with_base_url(api_key, "https://api.openai.com")
    }

    /// Create with a custom base URL (e.g., internal gateway, Azure OpenAI).
    pub fn with_base_url(
        api_key: impl Into<String>,
        base_url: impl Into<String>,
    ) -> NovaResult<Self> {
        let config = HttpClientConfig::new()
            .base_url(&base_url.into())
            .timeout_secs(60.0);

        let client = HttpClient::new(config)?;

        Ok(Self {
            client: Arc::new(client),
            api_key: api_key.into(),
            default_model: "gpt-4o".to_string(),
        })
    }

    pub fn with_default_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = model.into();
        self
    }

    fn convert_message(&self, msg: &Message) -> OpenAIMessage {
        let role = match msg.role {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::Tool => "tool",
        }
        .to_string();

        OpenAIMessage {
            role,
            content: Some(msg.content.clone()),
            name: msg.name.clone(),
            tool_calls: msg.tool_calls.as_ref().map(|calls| {
                calls
                    .iter()
                    .map(|c| OpenAIToolCall {
                        id: c.id.clone(),
                        r#type: "function".to_string(),
                        function: OpenAIFunctionCall {
                            name: c.name.clone(),
                            arguments: c.arguments.to_string(),
                        },
                    })
                    .collect()
            }),
            tool_call_id: msg.tool_call_id.clone(),
        }
    }

    fn convert_tool(&self, tool: &ToolDefinition) -> OpenAITool {
        OpenAITool {
            r#type: "function".to_string(),
            function: OpenAIFunction {
                name: tool.name.clone(),
                description: tool.description.clone(),
                parameters: tool.parameters.clone(),
            },
        }
    }

    fn build_request(&self, url: &str, body: serde_json::Value) -> RequestBuilder {
        RequestBuilder::new(HttpMethod::Post, url)
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json_value(body)
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    fn provider_name(&self) -> &str {
        "openai"
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "gpt-4o".to_string(),
            "gpt-4o-mini".to_string(),
            "gpt-4-turbo".to_string(),
            "gpt-4".to_string(),
            "gpt-3.5-turbo".to_string(),
            "o1".to_string(),
            "o1-mini".to_string(),
        ]
    }

    async fn complete(&self, request: CompletionRequest) -> NovaResult<CompletionResponse> {
        let model = if request.model.is_empty() {
            self.default_model.clone()
        } else {
            request.model.clone()
        };

        debug!("OpenAI completion request: model={}", model);

        let messages: Vec<OpenAIMessage> = request
            .messages
            .iter()
            .map(|m| self.convert_message(m))
            .collect();

        let tools: Option<Vec<OpenAITool>> = request
            .tools
            .as_ref()
            .map(|tools| tools.iter().map(|t| self.convert_tool(t)).collect());

        let mut body = serde_json::json!({
            "model": model,
            "messages": messages,
        });

        if let Some(temp) = request.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }

        if let Some(tools) = tools {
            body["tools"] = serde_json::to_value(tools)?;
        }

        // Structured output: map response_schema → response_format (native) or extras fallback
        if let Some(schema) = &request.response_schema {
            body["response_format"] = serde_json::json!({
                "type": "json_schema",
                "json_schema": {
                    "name": "structured_output",
                    "strict": true,
                    "schema": schema
                }
            });
        } else if let Some(rf) = request.extras.get("response_format") {
            body["response_format"] = rf.clone();
        }

        let builder = self.build_request("/v1/chat/completions", body);
        let response = self.client.execute_builder(builder).await?;

        if !response.is_success() {
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(NovaError::ApiError(format!(
                "OpenAI API error ({}): {}",
                response.status_code, error_text
            )));
        }

        let response_text = response.text()?;
        let openai_response: OpenAIResponse = serde_json::from_str(&response_text)?;

        let choice = openai_response
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| NovaError::ApiError("No choices in response".to_string()))?;

        let tool_calls: Option<Vec<ToolCall>> = choice.message.tool_calls.map(|calls| {
            calls
                .into_iter()
                .map(|c| ToolCall {
                    id: c.id,
                    name: c.function.name,
                    arguments: serde_json::from_str(&c.function.arguments)
                        .unwrap_or(serde_json::json!({})),
                })
                .collect()
        });

        Ok(CompletionResponse {
            content: choice.message.content.unwrap_or_default(),
            finish_reason: choice.finish_reason,
            model: openai_response.model,
            tool_calls,
            usage: TokenUsage {
                prompt_tokens: openai_response.usage.prompt_tokens,
                completion_tokens: openai_response.usage.completion_tokens,
                total_tokens: openai_response.usage.total_tokens,
            },
            metadata: HashMap::new(),
        })
    }

    async fn complete_stream(&self, _request: CompletionRequest) -> NovaResult<StreamResponse> {
        // TODO: Implement streaming for OpenAI
        Err(NovaError::InvalidRequest(
            "Streaming not yet implemented for OpenAI".to_string(),
        ))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAIToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIToolCall {
    id: String,
    r#type: String,
    function: OpenAIFunctionCall,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAITool {
    r#type: String,
    function: OpenAIFunction,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIFunction {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    #[allow(dead_code)]
    id: String,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_name() {
        let provider = OpenAIProvider::new("test-key").unwrap();
        assert_eq!(provider.provider_name(), "openai");
    }

    #[test]
    fn test_supported_models() {
        let provider = OpenAIProvider::new("test-key").unwrap();
        let models = provider.supported_models();
        assert!(models.contains(&"gpt-4o".to_string()));
    }
}
