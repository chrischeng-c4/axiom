//! Opaque types for the `agent-mamba` FFI layer.
//!
//! HANDWRITE-BEGIN reason: codegen has no generator for mamba FFI
//! wrapper types yet — the typed-I/O track will close this gap
//! (issue link TBD; tracked under the agent-track epic).

use agent::Schema as CoreSchema;

/// Opaque mamba handle for an in-progress object schema builder.
///
/// Mirrors `agent::ObjectSchemaBuilder` but uses owned-`Box`ed sub-schemas
/// (rather than the core's `Vec<(String, Schema)>`) so that the mamba FFI
/// can hand out `MbSchema` pointers to its callers without lifetime games.
#[derive(Debug, Default, Clone)]
pub struct MbSchemaBuilder {
    pub properties: Vec<(String, CoreSchema)>,
    pub required: Vec<String>,
}

impl MbSchemaBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn field(&mut self, name: impl Into<String>, schema: CoreSchema) {
        self.properties.push((name.into(), schema));
    }

    pub fn required(&mut self, name: impl Into<String>) {
        self.required.push(name.into());
    }

    /// Convert into a finalized [`MbSchema`] backed by [`agent::Schema::Object`].
    pub fn into_object(self) -> MbSchema {
        MbSchema(CoreSchema::Object {
            properties: self.properties,
            required: self.required,
        })
    }
}

/// Opaque mamba handle for a finalized [`agent::Schema`].
#[derive(Debug, Clone)]
pub struct MbSchema(pub CoreSchema);

// HANDWRITE-END

/// An agent builder used to configure and construct an [`MbLlmAgent`].
#[derive(Debug, Clone, Default)]
pub struct MbAgentBuilder {
    /// LLM provider name (e.g. "claude", "gemini", "openai").
    pub provider_name: String,
    /// API key for the provider.
    pub api_key: String,
    /// Optional system prompt.
    pub system_prompt: String,
}

impl MbAgentBuilder {
    pub fn new() -> Self {
        Self::default()
    }
}

/// A constructed LLM agent ready to run prompts.
#[derive(Debug, Clone)]
pub struct MbLlmAgent {
    /// LLM provider name (e.g. "claude", "gemini", "openai").
    pub provider_name: String,
    /// API key for the provider.
    pub api_key: String,
    /// Optional system prompt.
    pub system_prompt: String,
}

/// An LLM provider handle (stores name + api key).
#[derive(Debug, Clone)]
pub struct MbProvider {
    /// Provider name (e.g. "claude", "gemini", "openai").
    pub name: String,
    /// API key.
    pub api_key: String,
}

impl MbProvider {
    pub fn new(name: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self { name: name.into(), api_key: api_key.into() }
    }
}

/// A single chat message (role + content).
#[derive(Debug, Clone)]
pub struct MbMessage {
    /// Role: "user", "assistant", or "system".
    pub role: String,
    /// Message content text.
    pub content: String,
}

impl MbMessage {
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self { role: role.into(), content: content.into() }
    }
}

/// A governed agent team (#1545). Holds an ordered list of role names that
/// will be invoked when `team_run` is called. Role-name strings match the
/// serde-`lowercase` form of `cclab_agent::types::AgentRole`
/// (`pm | designer | dev | data | qa | release`).
#[derive(Debug, Clone, Default)]
pub struct MbAgentTeam {
    pub roles: Vec<String>,
}

impl MbAgentTeam {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_role(&mut self, role: impl Into<String>) {
        self.roles.push(role.into());
    }
}

/// A registry of named tools (Mamba function pointers).
#[derive(Debug, Clone, Default)]
pub struct MbToolRegistry {
    /// Registered tools as `(tool_name, func_ptr)` pairs.
    pub tools: Vec<(String, usize)>,
}

impl MbToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, name: impl Into<String>, func_ptr: usize) {
        self.tools.push((name.into(), func_ptr));
    }
}
