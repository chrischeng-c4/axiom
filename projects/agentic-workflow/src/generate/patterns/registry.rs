// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/patterns/registry.md#source
// CODEGEN-BEGIN
//! Built-in UX pattern registry.
//!
//! Adding a new pattern requires a code change — same principle as the
//! design system registry in tech_stack.
//!
//! Pattern definitions will be added in a future change.

use super::UxPattern;

/// Built-in pattern registry. Currently empty — patterns added incrementally.
/// @spec projects/agentic-workflow/tech-design/core/generate/patterns/registry.md#source
pub fn pattern_registry() -> &'static [UxPattern] {
    static REGISTRY: &[UxPattern] = &[];
    REGISTRY
}

/// Alias for backward compatibility
pub const PATTERN_REGISTRY: &[UxPattern] = &[];

// CODEGEN-END
