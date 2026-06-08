//! UX Pattern Library — design-system-agnostic layout patterns.
//!
//! Patterns are abstract layout recipes (e.g., `dashboard-with-drawer`, `crud-table`)
//! that wireframe specs can reference by ID instead of describing full layout structure.
//!
//! Status: Extension point defined — pattern content deferred.

pub mod registry;
pub mod resolver;

use std::collections::HashMap;

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/patterns/types.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// A node in the abstract layout tree.
/// @spec projects/agentic-workflow/tech-design/core/generate/patterns/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternNode {
    pub kind: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub slot_ref: Option<String>,
    #[serde(default)]
    pub props: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub children: Vec<PatternNode>,
}

/// A named insertion point in the pattern's layout tree.
/// @spec projects/agentic-workflow/tech-design/core/generate/patterns/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternSlot {
    pub name: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub description: String,
}

/// Content provided by a wireframe to fill a pattern slot.
/// @spec projects/agentic-workflow/tech-design/core/generate/patterns/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotContent {
    pub component: String,
    #[serde(default)]
    pub props: HashMap<String, serde_json::Value>,
}

/// A reusable layout recipe that wireframe specs reference by ID.
/// @spec projects/agentic-workflow/tech-design/core/generate/patterns/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UxPattern {
    pub id: String,
    pub name: String,
    pub description: String,
    pub slots: Vec<PatternSlot>,
    pub layout: Vec<PatternNode>,
}
// CODEGEN-END
/// Extension point for external pattern sources (deferred implementation).
///
/// When implemented, this trait allows custom pattern providers beyond the built-in registry.
/// Built-in registry is checked first; external sources are fallback.
pub trait PatternSource {
    fn get_pattern(&self, id: &str) -> Option<&UxPattern>;
}

pub use registry::PATTERN_REGISTRY;
pub use resolver::{expand_pattern, resolve_pattern};
