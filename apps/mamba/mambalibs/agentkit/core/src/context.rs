use crate::error::NovaResult;
use crate::llm::{CompletionRequest, LLMProvider};
use crate::tokenizer::{self, Tokenizer};
use crate::types::{Message, Role};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

const DEFAULT_RESERVED_TOKENS: u32 = 4096;
const DEFAULT_COMPACT_THRESHOLD: f64 = 0.80;
const DEFAULT_KEEP_RECENT: usize = 4;
const DEFAULT_COMPACT_MODEL: &str = "claude-3-haiku-20240307";

/// Context manager for conversation history and token budget tracking
pub struct ContextManager {
    messages: Vec<Message>,
    system_prompt: Option<String>,
    max_tokens: u32,
    current_tokens: u32,
    reserved_tokens: u32,
    metadata: HashMap<String, serde_json::Value>,
    tokenizer: Box<dyn Tokenizer>,
    pub model: String,
    /// Provider for LLM-based summarization during compaction
    compact_provider: Option<Arc<dyn LLMProvider>>,
    /// Model to use for summarization (default: haiku)
    compact_model: String,
    /// Fraction of max_tokens that triggers compaction (default: 0.80)
    compact_threshold: f64,
    /// Number of recent messages to always keep (default: 4)
    keep_recent: usize,
}

impl ContextManager {
    /// Create a new context manager for the given model
    pub fn new(max_tokens: u32) -> Self {
        Self {
            messages: Vec::new(),
            system_prompt: None,
            max_tokens,
            current_tokens: 0,
            reserved_tokens: DEFAULT_RESERVED_TOKENS,
            metadata: HashMap::new(),
            tokenizer: Box::new(tokenizer::EstimateTokenizer::new()),
            model: String::new(),
            compact_provider: None,
            compact_model: DEFAULT_COMPACT_MODEL.to_string(),
            compact_threshold: DEFAULT_COMPACT_THRESHOLD,
            keep_recent: DEFAULT_KEEP_RECENT,
        }
    }

    /// Create for a specific model (uses accurate tokenizer if available)
    pub fn for_model(model: &str, max_tokens: u32) -> Self {
        Self {
            tokenizer: tokenizer::tokenizer_for_model(model),
            model: model.to_string(),
            ..Self::new(max_tokens)
        }
    }

    /// Create with default token limit (128k)
    pub fn default_128k() -> Self {
        Self::new(128_000)
    }

    /// Create with smaller token limit (32k)
    pub fn default_32k() -> Self {
        Self::new(32_000)
    }

    /// Set the LLM provider for smart compaction summarization
    pub fn set_compact_provider(&mut self, provider: Arc<dyn LLMProvider>) {
        self.compact_provider = Some(provider);
    }

    /// Set the model used for summarization
    pub fn set_compact_model(&mut self, model: impl Into<String>) {
        self.compact_model = model.into();
    }

    /// Set the system prompt
    pub fn set_system_prompt(&mut self, prompt: impl Into<String>) {
        let prompt = prompt.into();
        let new_tokens = self.tokenizer.count(&prompt);
        let old_tokens = self
            .system_prompt
            .as_ref()
            .map_or(0, |p| self.tokenizer.count(p));
        self.current_tokens =
            (self.current_tokens as i64 + new_tokens as i64 - old_tokens as i64) as u32;
        self.system_prompt = Some(prompt);
    }

    /// Add a message to the context
    pub fn add_message(&mut self, message: Message) {
        let tokens = self.estimate_message_tokens(&message);
        self.current_tokens += tokens;
        self.messages.push(message);

        if self.needs_compression() {
            self.compress();
        }
    }

    /// Add a user message
    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.add_message(Message::user(content));
    }

    /// Add an assistant message
    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        self.add_message(Message::assistant(content));
    }

    /// Add a tool result message
    pub fn add_tool_result(&mut self, tool_call_id: impl Into<String>, content: impl Into<String>) {
        self.add_message(Message::tool(tool_call_id, content));
    }

    /// Get all messages for LLM request
    pub fn get_messages(&self) -> Vec<Message> {
        let mut messages = Vec::new();
        if let Some(ref prompt) = self.system_prompt {
            messages.push(Message::system(prompt.clone()));
        }
        messages.extend(self.messages.clone());
        messages
    }

    /// Get available token budget for response
    pub fn available_tokens(&self) -> u32 {
        self.max_tokens
            .saturating_sub(self.current_tokens)
            .saturating_sub(self.reserved_tokens)
    }

    fn needs_compression(&self) -> bool {
        let threshold = (self.max_tokens as f64 * self.compact_threshold) as u32;
        self.current_tokens > threshold
    }

    /// Compress context. Tries LLM summarization first, falls back to FIFO.
    fn compress(&mut self) {
        if self.compact_provider.is_some() {
            if let Ok(()) = self.compress_with_summarization() {
                return;
            }
        }
        self.compress_fifo();
    }

    /// FIFO compression: remove oldest messages, respecting tool-call pairing
    fn compress_fifo(&mut self) {
        // Target 60% of max to leave headroom after compaction
        let target_tokens = (self.max_tokens as f64 * 0.60) as u32;

        while self.current_tokens > target_tokens && self.messages.len() > self.keep_recent {
            // Find the next removable chunk (respecting tool-call pairing)
            let remove_count = self.paired_remove_count(0);
            let mut removed_tokens = 0u32;
            for i in 0..remove_count {
                if i < self.messages.len() {
                    removed_tokens += self.estimate_message_tokens(&self.messages[i]);
                }
            }
            for _ in 0..remove_count.min(self.messages.len()) {
                self.messages.remove(0);
            }
            self.current_tokens = self.current_tokens.saturating_sub(removed_tokens);
        }
    }

    /// Determine how many messages to remove starting at `idx` to keep tool-call pairs intact
    fn paired_remove_count(&self, idx: usize) -> usize {
        if idx >= self.messages.len() {
            return 0;
        }
        let msg = &self.messages[idx];

        // If this is an assistant message with tool_calls, also remove subsequent tool results
        if msg.role == Role::Assistant && msg.tool_calls.is_some() {
            let tool_calls = msg.tool_calls.as_ref().unwrap();
            let tool_call_ids: Vec<&str> = tool_calls.iter().map(|c| c.id.as_str()).collect();

            let mut count = 1; // the assistant message itself
            for j in (idx + 1)..self.messages.len() {
                if self.messages[j].role == Role::Tool {
                    if let Some(ref tcid) = self.messages[j].tool_call_id {
                        if tool_call_ids.contains(&tcid.as_str()) {
                            count += 1;
                            continue;
                        }
                    }
                }
                break;
            }
            count
        } else if msg.role == Role::Tool {
            // Don't remove a tool result without its assistant message
            // (this shouldn't happen if we always start from the oldest)
            1
        } else {
            1
        }
    }

    /// LLM-based summarization compression
    fn compress_with_summarization(&mut self) -> NovaResult<()> {
        let provider = self.compact_provider.as_ref().unwrap().clone();
        let model = self.compact_model.clone();

        // Identify compressible messages (skip recent, keep system prompt separate)
        let total = self.messages.len();
        if total <= self.keep_recent {
            return Ok(());
        }
        let compress_end = total - self.keep_recent;

        // Collect messages to summarize
        let to_summarize: Vec<&Message> = self.messages[..compress_end].iter().collect();
        if to_summarize.is_empty() {
            return Ok(());
        }

        // Build summarization prompt
        let conversation_text: String = to_summarize
            .iter()
            .map(|m| format!("[{:?}]: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        let summary_prompt = format!(
            "Summarize this conversation concisely, preserving:\n\
             - Key decisions made\n\
             - Important variables/values mentioned\n\
             - Goals and current state\n\
             - Tool call results (outcomes only, not full output)\n\n\
             Conversation:\n{}\n\n\
             Provide a concise summary:",
            conversation_text
        );

        // Execute summarization synchronously using tokio
        let request = CompletionRequest::new(vec![Message::user(summary_prompt)], &model)
            .with_max_tokens(1024);

        let response = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(provider.complete(request))
        })?;

        // Replace old messages with summary
        let summary_message = Message::system(format!("[Context Summary]: {}", response.content));

        let kept_messages: Vec<Message> = self.messages[compress_end..].to_vec();
        self.messages.clear();
        self.messages.push(summary_message);
        self.messages.extend(kept_messages);

        // Recalculate tokens
        self.recalculate_tokens();

        Ok(())
    }

    fn recalculate_tokens(&mut self) {
        self.current_tokens = self
            .system_prompt
            .as_ref()
            .map_or(0, |p| self.tokenizer.count(p));
        for msg in &self.messages {
            self.current_tokens += self.estimate_message_tokens(msg);
        }
    }

    fn estimate_message_tokens(&self, message: &Message) -> u32 {
        let base = self.tokenizer.count(&message.content);
        let tool_calls = message
            .tool_calls
            .as_ref()
            .map(|calls| {
                calls
                    .iter()
                    .map(|c| {
                        self.tokenizer.count(&c.name)
                            + self.tokenizer.count(&c.arguments.to_string())
                    })
                    .sum::<u32>()
            })
            .unwrap_or(0);

        base + tool_calls + 4 // 4 = per-message overhead (role, separators)
    }

    /// Clear all messages (keep system prompt)
    pub fn clear(&mut self) {
        self.messages.clear();
        self.current_tokens = self
            .system_prompt
            .as_ref()
            .map_or(0, |p| self.tokenizer.count(p));
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Get estimated current token count
    pub fn current_token_count(&self) -> u32 {
        self.current_tokens
    }

    /// Set metadata
    pub fn set_metadata(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.metadata.insert(key.into(), value);
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Get current context statistics
    pub fn stats(&self) -> ContextStats {
        ContextStats {
            message_count: self.messages.len(),
            estimated_tokens: self.current_tokens,
            max_tokens: self.max_tokens,
            available_tokens: self.available_tokens(),
            compression_triggered: false,
        }
    }
}

/// Statistics about context usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextStats {
    pub message_count: usize,
    pub estimated_tokens: u32,
    pub max_tokens: u32,
    pub available_tokens: u32,
    pub compression_triggered: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_manager_creation() {
        let ctx = ContextManager::new(10000);
        assert_eq!(ctx.message_count(), 0);
        assert!(ctx.available_tokens() > 0);
    }

    #[test]
    fn test_add_messages() {
        let mut ctx = ContextManager::new(10000);
        ctx.add_user_message("Hello");
        ctx.add_assistant_message("Hi there!");

        assert_eq!(ctx.message_count(), 2);
        let messages = ctx.get_messages();
        assert_eq!(messages.len(), 2);
    }

    #[test]
    fn test_system_prompt() {
        let mut ctx = ContextManager::new(10000);
        ctx.set_system_prompt("You are a helpful assistant.");
        ctx.add_user_message("Hello");

        let messages = ctx.get_messages();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, Role::System);
    }

    #[test]
    fn test_clear() {
        let mut ctx = ContextManager::new(10000);
        ctx.set_system_prompt("System");
        ctx.add_user_message("Hello");
        ctx.add_assistant_message("Hi");

        ctx.clear();
        assert_eq!(ctx.message_count(), 0);

        let messages = ctx.get_messages();
        assert_eq!(messages.len(), 1); // system prompt only
    }

    #[test]
    fn test_for_model_openai() {
        let ctx = ContextManager::for_model("gpt-4o", 128_000);
        assert_eq!(ctx.model, "gpt-4o");
    }

    #[test]
    fn test_compress_fifo_keeps_recent() {
        // Use a budget small enough to trigger compression but > reserved_tokens
        let mut ctx = ContextManager::new(10_000);
        ctx.reserved_tokens = 100;

        // Add many large messages to exceed 80% threshold (8000 tokens)
        // Each message ~250 chars = ~62 tokens + 4 overhead = ~66 tokens
        // 130 messages * 66 tokens ≈ 8580 tokens, exceeding 8000 threshold
        let big_msg = "x".repeat(250);
        for _ in 0..130 {
            ctx.add_user_message(&big_msg);
        }

        // After compression, should have fewer than 130 messages
        assert!(
            ctx.message_count() < 130,
            "expected <130 but got {}",
            ctx.message_count()
        );
        // Should keep at least keep_recent messages
        assert!(ctx.message_count() >= ctx.keep_recent);
    }

    #[test]
    fn test_tool_call_pairing() {
        let _ctx = ContextManager::new(10000);

        // Create an assistant message with tool calls
        let mut assistant_msg = Message::assistant("Let me check.");
        assistant_msg.tool_calls = Some(vec![crate::types::ToolCall {
            id: "call_1".to_string(),
            name: "read_file".to_string(),
            arguments: serde_json::json!({"path": "test.txt"}),
        }]);

        let messages = vec![
            Message::user("Hello"),
            assistant_msg,
            Message::tool("call_1", "file contents here"),
            Message::user("Thanks"),
        ];

        // Check paired_remove_count
        let mut test_ctx = ContextManager::new(10000);
        for m in messages {
            test_ctx.messages.push(m);
        }

        // At idx 1 (assistant with tool_calls), should remove 2 (assistant + tool result)
        assert_eq!(test_ctx.paired_remove_count(1), 2);
        // At idx 0 (plain user message), should remove 1
        assert_eq!(test_ctx.paired_remove_count(0), 1);
    }
}
