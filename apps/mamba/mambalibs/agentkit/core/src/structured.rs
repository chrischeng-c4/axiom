use crate::error::{NovaError, NovaResult};
use crate::llm::{CompletionRequest, CompletionResponse, LLMProvider, ToolDefinition};
use crate::types::Message;
use serde_json::Value;

/// Validate a JSON value against a JSON Schema
pub fn validate_json(value: &Value, schema: &Value) -> NovaResult<()> {
    let validator = jsonschema::validator_for(schema)
        .map_err(|e| NovaError::SchemaValidationError(format!("Invalid schema: {}", e)))?;

    if let Err(err) = validator.validate(value) {
        return Err(NovaError::SchemaValidationError(err.to_string()));
    }
    Ok(())
}

/// Inject response_schema into a CompletionRequest based on provider type.
/// Returns the modified request and whether tool extraction is needed.
pub fn inject_schema_for_provider(
    mut request: CompletionRequest,
    schema: &Value,
    provider_name: &str,
) -> (CompletionRequest, bool) {
    match provider_name {
        "openai" => {
            // OpenAI: use response_format with json_schema
            request.extras.insert(
                "response_format".to_string(),
                serde_json::json!({
                    "type": "json_schema",
                    "json_schema": {
                        "name": "structured_output",
                        "strict": true,
                        "schema": schema,
                    }
                }),
            );
            (request, false)
        }
        "anthropic" => {
            // Claude: inject schema as a single tool definition
            let tool = ToolDefinition {
                name: "__structured_output".to_string(),
                description: "Extract structured data matching the required schema. \
                    You MUST call this tool with the extracted data."
                    .to_string(),
                parameters: schema.clone(),
            };
            let mut tools = request.tools.unwrap_or_default();
            tools.push(tool);
            request.tools = Some(tools);
            (request, true) // need to extract from tool_use
        }
        "google" => {
            // Gemini: set response_mime_type + response_schema in generationConfig
            request.extras.insert(
                "response_mime_type".to_string(),
                serde_json::json!("application/json"),
            );
            request
                .extras
                .insert("response_schema".to_string(), schema.clone());
            (request, false)
        }
        _ => (request, false),
    }
}

/// Extract structured output from a response (handles Claude tool-use pattern)
pub fn extract_structured_content(
    response: &CompletionResponse,
    use_tool_extraction: bool,
) -> NovaResult<Value> {
    if use_tool_extraction {
        // Claude: extract from tool_use block
        if let Some(tool_calls) = &response.tool_calls {
            for call in tool_calls {
                if call.name == "__structured_output" {
                    return Ok(call.arguments.clone());
                }
            }
        }
        // Fallback: try parsing content as JSON
        serde_json::from_str(&response.content).map_err(|e| {
            NovaError::SchemaValidationError(format!("No structured output found: {}", e))
        })
    } else {
        // OpenAI/Gemini: parse content directly
        serde_json::from_str(&response.content).map_err(|e| {
            NovaError::SchemaValidationError(format!("Invalid JSON in response: {}", e))
        })
    }
}

/// Complete with structured output, with auto-retry on validation failure
pub async fn complete_structured(
    provider: &dyn LLMProvider,
    request: CompletionRequest,
    schema: &Value,
    max_retries: u32,
) -> NovaResult<(CompletionResponse, Value)> {
    let (injected_request, use_tool_extraction) =
        inject_schema_for_provider(request.clone(), schema, provider.provider_name());

    let mut current_request = injected_request;
    let mut last_error = String::new();

    for attempt in 0..=max_retries {
        let response = provider.complete(current_request.clone()).await?;
        let extracted = extract_structured_content(&response, use_tool_extraction);

        match extracted {
            Ok(value) => {
                // Validate against schema
                match validate_json(&value, schema) {
                    Ok(()) => {
                        let mut final_response = response;
                        final_response.content = serde_json::to_string(&value).unwrap_or_default();
                        return Ok((final_response, value));
                    }
                    Err(NovaError::SchemaValidationError(msg)) => {
                        last_error = msg;
                    }
                    Err(e) => return Err(e),
                }
            }
            Err(NovaError::SchemaValidationError(msg)) => {
                last_error = msg;
            }
            Err(e) => return Err(e),
        }

        if attempt < max_retries {
            // Append error feedback for retry
            current_request.messages.push(Message::user(format!(
                "Your previous response had a validation error: {}. \
                 Please fix and try again. The output must conform to the JSON schema.",
                last_error
            )));
        }
    }

    Err(NovaError::SchemaValidationError(format!(
        "Failed after {} retries. Last error: {}",
        max_retries, last_error
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_json_pass() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            },
            "required": ["name"]
        });
        let value = serde_json::json!({"name": "John"});
        assert!(validate_json(&value, &schema).is_ok());
    }

    #[test]
    fn test_validate_json_fail() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            },
            "required": ["name"]
        });
        let value = serde_json::json!({"age": 30});
        assert!(validate_json(&value, &schema).is_err());
    }

    #[test]
    fn test_inject_schema_openai() {
        let request = CompletionRequest::new(vec![], "gpt-4o");
        let schema = serde_json::json!({"type": "object"});
        let (req, extract) = inject_schema_for_provider(request, &schema, "openai");
        assert!(!extract);
        assert!(req.extras.contains_key("response_format"));
    }

    #[test]
    fn test_inject_schema_claude() {
        let request = CompletionRequest::new(vec![], "claude-sonnet-4-20250514");
        let schema = serde_json::json!({"type": "object"});
        let (req, extract) = inject_schema_for_provider(request, &schema, "anthropic");
        assert!(extract);
        assert!(req.tools.is_some());
        let tools = req.tools.unwrap();
        assert!(tools.iter().any(|t| t.name == "__structured_output"));
    }

    #[test]
    fn test_inject_schema_gemini() {
        let request = CompletionRequest::new(vec![], "gemini-2.0-flash");
        let schema = serde_json::json!({"type": "object"});
        let (req, extract) = inject_schema_for_provider(request, &schema, "google");
        assert!(!extract);
        assert!(req.extras.contains_key("response_mime_type"));
        assert!(req.extras.contains_key("response_schema"));
    }

    #[test]
    fn test_extract_from_tool_call() {
        let response = CompletionResponse {
            content: String::new(),
            tool_calls: Some(vec![crate::types::ToolCall {
                id: "1".to_string(),
                name: "__structured_output".to_string(),
                arguments: serde_json::json!({"name": "John"}),
            }]),
            finish_reason: "tool_calls".to_string(),
            usage: crate::types::TokenUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            },
            model: "test".to_string(),
            metadata: Default::default(),
        };
        let result = extract_structured_content(&response, true).unwrap();
        assert_eq!(result["name"], "John");
    }
}
