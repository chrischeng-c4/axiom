use crate::error::{NovaError, NovaResult};
use crate::types::Message;
use tiktoken_rs::CoreBPE;

/// Trait for token counting strategies
pub trait Tokenizer: Send + Sync {
    fn count(&self, text: &str) -> u32;
    fn truncate(&self, text: &str, max_tokens: u32) -> String;
}

/// BPE tokenizer using tiktoken-rs for OpenAI models
pub struct TiktokenTokenizer {
    bpe: CoreBPE,
}

impl TiktokenTokenizer {
    pub fn for_model(model: &str) -> NovaResult<Self> {
        let bpe = tiktoken_rs::get_bpe_from_model(model).map_err(|e| {
            NovaError::ConfigError(format!(
                "Failed to load tokenizer for model '{}': {}",
                model, e
            ))
        })?;
        Ok(Self { bpe })
    }

    pub fn cl100k_base() -> NovaResult<Self> {
        let bpe = tiktoken_rs::cl100k_base().map_err(|e| {
            NovaError::ConfigError(format!("Failed to load cl100k_base tokenizer: {}", e))
        })?;
        Ok(Self { bpe })
    }

    pub fn o200k_base() -> NovaResult<Self> {
        let bpe = tiktoken_rs::o200k_base().map_err(|e| {
            NovaError::ConfigError(format!("Failed to load o200k_base tokenizer: {}", e))
        })?;
        Ok(Self { bpe })
    }
}

impl Tokenizer for TiktokenTokenizer {
    fn count(&self, text: &str) -> u32 {
        self.bpe.encode_with_special_tokens(text).len() as u32
    }

    fn truncate(&self, text: &str, max_tokens: u32) -> String {
        let tokens = self.bpe.encode_with_special_tokens(text);
        if tokens.len() <= max_tokens as usize {
            return text.to_string();
        }
        let truncated = &tokens[..max_tokens as usize];
        self.bpe.decode(truncated.to_vec()).unwrap_or_default()
    }
}

/// Estimation-based tokenizer (chars/4 heuristic)
/// Used for Claude/Gemini models that don't publish tokenizers
pub struct EstimateTokenizer {
    chars_per_token: u32,
}

impl EstimateTokenizer {
    pub fn new() -> Self {
        Self { chars_per_token: 4 }
    }
}

impl Default for EstimateTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for EstimateTokenizer {
    fn count(&self, text: &str) -> u32 {
        (text.len() as u32 / self.chars_per_token).max(1)
    }

    fn truncate(&self, text: &str, max_tokens: u32) -> String {
        let max_chars = (max_tokens * self.chars_per_token) as usize;
        if text.len() <= max_chars {
            return text.to_string();
        }
        // Truncate at char boundary
        let mut end = max_chars;
        while end > 0 && !text.is_char_boundary(end) {
            end -= 1;
        }
        text[..end].to_string()
    }
}

/// Get the appropriate tokenizer for a model
pub fn tokenizer_for_model(model: &str) -> Box<dyn Tokenizer> {
    // OpenAI models: use tiktoken
    if model.starts_with("gpt-") || model.starts_with("o1") || model.starts_with("o3") {
        if let Ok(t) = TiktokenTokenizer::for_model(model) {
            return Box::new(t);
        }
    }
    // Claude, Gemini, and fallback: use estimation
    Box::new(EstimateTokenizer::new())
}

/// Count tokens for a single text string
pub fn count_tokens(text: &str, model: &str) -> u32 {
    tokenizer_for_model(model).count(text)
}

/// Count tokens for a list of messages (includes per-message overhead)
pub fn count_message_tokens(messages: &[Message], model: &str) -> u32 {
    let tokenizer = tokenizer_for_model(model);
    let per_message_overhead: u32 = 4; // role, separators

    messages
        .iter()
        .map(|msg| {
            let base = tokenizer.count(&msg.content);
            let tool_calls = msg
                .tool_calls
                .as_ref()
                .map(|calls| {
                    calls
                        .iter()
                        .map(|c| {
                            tokenizer.count(&c.name) + tokenizer.count(&c.arguments.to_string())
                        })
                        .sum::<u32>()
                })
                .unwrap_or(0);
            base + tool_calls + per_message_overhead
        })
        .sum()
}

/// Truncate text to fit within max_tokens
pub fn truncate(text: &str, max_tokens: u32, model: &str) -> String {
    tokenizer_for_model(model).truncate(text, max_tokens)
}

/// Model pricing per 1M tokens (input, output) in USD
fn model_pricing(model: &str) -> Option<(f64, f64)> {
    match model {
        "claude-sonnet-4-20250514" => Some((3.0, 15.0)),
        "claude-opus-4-20250514" => Some((15.0, 75.0)),
        "claude-3-5-sonnet-20241022" | "claude-3-5-sonnet-20240620" => Some((3.0, 15.0)),
        "claude-3-opus-20240229" => Some((15.0, 75.0)),
        "claude-3-haiku-20240307" => Some((0.25, 1.25)),
        "gpt-4o" => Some((2.50, 10.0)),
        "gpt-4o-mini" => Some((0.15, 0.60)),
        "gpt-4-turbo" => Some((10.0, 30.0)),
        "gemini-2.0-flash" => Some((0.10, 0.40)),
        "gemini-1.5-pro" => Some((1.25, 5.0)),
        _ => None,
    }
}

/// Estimate cost in USD for a given number of input tokens
pub fn estimate_cost(input_tokens: u32, output_tokens: u32, model: &str) -> f64 {
    let (input_price, output_price) = model_pricing(model).unwrap_or((1.0, 3.0));
    (input_tokens as f64 * input_price + output_tokens as f64 * output_price) / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_tokens_openai() {
        let count = count_tokens("Hello, world!", "gpt-4o");
        assert!(count > 0);
        assert!(count < 10); // "Hello, world!" is ~4 tokens
    }

    #[test]
    fn test_count_tokens_claude_fallback() {
        let count = count_tokens("Hello, world!", "claude-sonnet-4-20250514");
        assert_eq!(count, ("Hello, world!".len() as u32 / 4).max(1));
    }

    #[test]
    fn test_truncate_at_boundary() {
        let text = "Hello, this is a longer text that should be truncated at a token boundary.";
        let truncated = truncate(text, 5, "gpt-4o");
        let after_count = count_tokens(&truncated, "gpt-4o");
        assert!(after_count <= 5);
    }

    #[test]
    fn test_truncate_no_op() {
        let text = "Hi";
        let truncated = truncate(text, 100, "gpt-4o");
        assert_eq!(truncated, text);
    }

    #[test]
    fn test_estimate_cost() {
        let cost = estimate_cost(1000, 500, "gpt-4o");
        // gpt-4o: $2.50/1M input + $10.0/1M output
        let expected = (1000.0 * 2.50 + 500.0 * 10.0) / 1_000_000.0;
        assert!((cost - expected).abs() < 1e-10);
    }

    #[test]
    fn test_estimate_tokenizer() {
        let t = EstimateTokenizer::new();
        assert_eq!(t.count("abcdefgh"), 2); // 8 chars / 4 = 2
        assert_eq!(t.count("ab"), 1); // max(0, 1) = 1
    }

    #[test]
    fn test_count_message_tokens() {
        let messages = vec![Message::user("Hello"), Message::assistant("Hi there!")];
        let count = count_message_tokens(&messages, "gpt-4o");
        assert!(count > 0);
    }
}
