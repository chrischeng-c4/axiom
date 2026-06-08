// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/logic/defaults.md#source
// CODEGEN-BEGIN
//! Default model names and reasoning levels for LLM providers.
//! Change here to upgrade model versions across the codebase.

// Gemini
pub const GEMINI_FLASH_MODEL: &str = "gemini-3-flash-preview";
pub const GEMINI_PRO_MODEL: &str = "gemini-3.1-pro-preview";

// Codex — base model + reasoning tiers
pub const CODEX_MODEL: &str = "gpt-5.4";
pub const CODEX_REASONING_LOW: &str = "low";
pub const CODEX_REASONING_MEDIUM: &str = "medium";
pub const CODEX_REASONING_HIGH: &str = "high";
pub const CODEX_REASONING_XHIGH: &str = "xhigh";

// Codex Spark — lighter variant
pub const CODEX_SPARK_MODEL: &str = "gpt-5.4-mini";

// Claude
pub const CLAUDE_FAST_MODEL: &str = "claude-haiku-4-5";
pub const CLAUDE_BALANCED_MODEL: &str = "claude-sonnet-4-6";
pub const CLAUDE_DEEP_MODEL: &str = "claude-opus-4-6";
// CODEGEN-END
