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

/// Google Gemini LLM provider
pub struct GeminiProvider {
    client: Arc<HttpClient>,
    api_key: String,
    default_model: String,
    _base_url: String,
}

impl GeminiProvider {
    pub fn new(api_key: impl Into<String>) -> NovaResult<Self> {
        Self::with_base_url(api_key, "https://generativelanguage.googleapis.com")
    }

    /// Create with a custom base URL (e.g., internal gateway).
    ///
    /// ```rust,ignore
    /// let provider = GeminiProvider::with_base_url(
    ///     "your-api-key",
    ///     "https://your-internal-gateway.example.com/v1",
    /// )?;
    /// ```
    pub fn with_base_url(
        api_key: impl Into<String>,
        base_url: impl Into<String>,
    ) -> NovaResult<Self> {
        let base = base_url.into();
        let config = HttpClientConfig::new().base_url(&base).timeout_secs(60.0);

        let client = HttpClient::new(config)?;

        Ok(Self {
            client: Arc::new(client),
            api_key: api_key.into(),
            default_model: "gemini-2.0-flash".to_string(),
            _base_url: base,
        })
    }

    pub fn with_default_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = model.into();
        self
    }

    fn convert_messages(
        &self,
        messages: &[Message],
    ) -> (Option<GeminiSystemInstruction>, Vec<GeminiContent>) {
        let mut system_instruction = None;
        let mut contents = Vec::new();

        for msg in messages {
            match msg.role {
                Role::System => {
                    system_instruction = Some(GeminiSystemInstruction {
                        parts: vec![GeminiPart::Text {
                            text: msg.content.clone(),
                        }],
                    });
                }
                Role::User => {
                    contents.push(GeminiContent {
                        role: "user".to_string(),
                        parts: vec![GeminiPart::Text {
                            text: msg.content.clone(),
                        }],
                    });
                }
                Role::Assistant => {
                    let mut parts = vec![];

                    if !msg.content.is_empty() {
                        parts.push(GeminiPart::Text {
                            text: msg.content.clone(),
                        });
                    }

                    if let Some(tool_calls) = &msg.tool_calls {
                        for tool_call in tool_calls {
                            parts.push(GeminiPart::FunctionCall {
                                function_call: GeminiFunctionCall {
                                    name: tool_call.name.clone(),
                                    args: tool_call.arguments.clone(),
                                },
                            });
                        }
                    }

                    contents.push(GeminiContent {
                        role: "model".to_string(),
                        parts,
                    });
                }
                Role::Tool => {
                    contents.push(GeminiContent {
                        role: "user".to_string(),
                        parts: vec![GeminiPart::FunctionResponse {
                            function_response: GeminiFunctionResponse {
                                name: msg.tool_call_id.clone().unwrap_or_default(),
                                response: serde_json::json!({ "result": msg.content }),
                            },
                        }],
                    });
                }
            }
        }

        (system_instruction, contents)
    }

    fn convert_tools(&self, tools: &[ToolDefinition]) -> Vec<GeminiTool> {
        vec![GeminiTool {
            function_declarations: tools
                .iter()
                .map(|t| GeminiFunctionDeclaration {
                    name: t.name.clone(),
                    description: t.description.clone(),
                    parameters: t.parameters.clone(),
                })
                .collect(),
        }]
    }
}

#[async_trait]
impl LLMProvider for GeminiProvider {
    fn provider_name(&self) -> &str {
        "google"
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "gemini-2.0-flash".to_string(),
            "gemini-2.0-flash-lite".to_string(),
            "gemini-1.5-pro".to_string(),
            "gemini-1.5-flash".to_string(),
        ]
    }

    async fn complete(&self, request: CompletionRequest) -> NovaResult<CompletionResponse> {
        let model = if request.model.is_empty() {
            self.default_model.clone()
        } else {
            request.model.clone()
        };

        debug!("Gemini completion request: model={}", model);

        let (system_instruction, contents) = self.convert_messages(&request.messages);

        let mut body = serde_json::json!({
            "contents": contents,
        });

        if let Some(system) = system_instruction {
            body["systemInstruction"] = serde_json::to_value(system)?;
        }

        let mut generation_config = serde_json::Map::new();

        if let Some(temp) = request.temperature {
            generation_config.insert("temperature".to_string(), serde_json::json!(temp));
        }

        if let Some(max_tokens) = request.max_tokens {
            generation_config.insert("maxOutputTokens".to_string(), serde_json::json!(max_tokens));
        }

        // Structured output: map response_schema → responseMimeType + responseSchema
        if let Some(schema) = &request.response_schema {
            generation_config.insert(
                "responseMimeType".to_string(),
                serde_json::json!("application/json"),
            );
            generation_config.insert("responseSchema".to_string(), schema.clone());
        } else {
            // Fallback: check extras injected by complete_structured helper
            if let Some(mime_type) = request.extras.get("response_mime_type") {
                generation_config.insert("responseMimeType".to_string(), mime_type.clone());
            }
            if let Some(resp_schema) = request.extras.get("response_schema") {
                generation_config.insert("responseSchema".to_string(), resp_schema.clone());
            }
        }

        if !generation_config.is_empty() {
            body["generationConfig"] = serde_json::Value::Object(generation_config);
        }

        if let Some(tools) = &request.tools {
            body["tools"] = serde_json::to_value(self.convert_tools(tools))?;
        }

        let url = format!(
            "/v1beta/models/{}:generateContent?key={}",
            model, self.api_key
        );

        let builder = RequestBuilder::new(HttpMethod::Post, &url)
            .header("Content-Type", "application/json")
            .json_value(body);

        let response = self.client.execute_builder(builder).await?;

        if !response.is_success() {
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(NovaError::ApiError(format!(
                "Gemini API error ({}): {}",
                response.status_code, error_text
            )));
        }

        let response_text = response.text()?;
        let gemini_response: GeminiResponse = serde_json::from_str(&response_text)?;

        let candidate = gemini_response
            .candidates
            .into_iter()
            .next()
            .ok_or_else(|| NovaError::ApiError("No candidates in response".to_string()))?;

        let mut content = String::new();
        let mut tool_calls = Vec::new();

        for part in candidate.content.parts {
            match part {
                GeminiPart::Text { text } => {
                    if !content.is_empty() {
                        content.push('\n');
                    }
                    content.push_str(&text);
                }
                GeminiPart::FunctionCall { function_call } => {
                    tool_calls.push(ToolCall {
                        id: format!("call_{}", tool_calls.len()),
                        name: function_call.name,
                        arguments: function_call.args,
                    });
                }
                _ => {}
            }
        }

        let finish_reason = match candidate.finish_reason.as_deref() {
            Some("STOP") => "stop",
            Some("MAX_TOKENS") => "length",
            Some("SAFETY") => "content_filter",
            _ => "unknown",
        }
        .to_string();

        let usage = gemini_response.usage_metadata.unwrap_or_default();

        Ok(CompletionResponse {
            content,
            finish_reason,
            model,
            tool_calls: if tool_calls.is_empty() {
                None
            } else {
                Some(tool_calls)
            },
            usage: TokenUsage {
                prompt_tokens: usage.prompt_token_count,
                completion_tokens: usage.candidates_token_count,
                total_tokens: usage.total_token_count,
            },
            metadata: HashMap::new(),
        })
    }

    async fn complete_stream(&self, _request: CompletionRequest) -> NovaResult<StreamResponse> {
        Err(NovaError::InvalidRequest(
            "Streaming not yet implemented for Gemini".to_string(),
        ))
    }
}

#[derive(Debug, Serialize)]
struct GeminiSystemInstruction {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum GeminiPart {
    Text {
        text: String,
    },
    FunctionCall {
        #[serde(rename = "functionCall")]
        function_call: GeminiFunctionCall,
    },
    FunctionResponse {
        #[serde(rename = "functionResponse")]
        function_response: GeminiFunctionResponse,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiFunctionCall {
    name: String,
    args: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiFunctionResponse {
    name: String,
    response: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct GeminiTool {
    #[serde(rename = "functionDeclarations")]
    function_declarations: Vec<GeminiFunctionDeclaration>,
}

#[derive(Debug, Serialize)]
struct GeminiFunctionDeclaration {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<GeminiUsageMetadata>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiCandidateContent,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidateContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Deserialize, Default)]
struct GeminiUsageMetadata {
    #[serde(rename = "promptTokenCount", default)]
    prompt_token_count: u32,
    #[serde(rename = "candidatesTokenCount", default)]
    candidates_token_count: u32,
    #[serde(rename = "totalTokenCount", default)]
    total_token_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_name() {
        let provider = GeminiProvider::new("test-key").unwrap();
        assert_eq!(provider.provider_name(), "google");
    }

    #[test]
    fn test_supported_models() {
        let provider = GeminiProvider::new("test-key").unwrap();
        let models = provider.supported_models();
        assert!(models.contains(&"gemini-2.0-flash".to_string()));
    }
}
