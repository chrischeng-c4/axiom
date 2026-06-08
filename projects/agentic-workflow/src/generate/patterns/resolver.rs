// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/patterns/resolver.md#source
// CODEGEN-BEGIN
//! Pattern resolution — expand pattern references into full layout trees.

use super::{PatternNode, SlotContent, UxPattern};
use std::collections::HashMap;

/// Look up a pattern by ID from the built-in registry.
/// @spec projects/agentic-workflow/tech-design/core/generate/patterns/resolver.md#source
pub fn resolve_pattern(pattern_id: &str) -> Option<&'static UxPattern> {
    super::registry::PATTERN_REGISTRY
        .iter()
        .find(|p| p.id == pattern_id)
}

/// Expand a pattern into a full layout tree with slots filled from content map.
///
/// Returns the expanded layout tree and any validation warnings.
/// @spec projects/agentic-workflow/tech-design/core/generate/patterns/resolver.md#source
pub fn expand_pattern(
    pattern: &UxPattern,
    slot_contents: &HashMap<String, SlotContent>,
) -> (Vec<PatternNode>, Vec<String>) {
    let mut warnings = Vec::new();

    // Check for unfilled required slots
    for slot in &pattern.slots {
        if slot.required && !slot_contents.contains_key(&slot.name) {
            warnings.push(format!(
                "required slot '{}' in pattern '{}' was not filled",
                slot.name, pattern.id
            ));
        }
    }

    let expanded = pattern
        .layout
        .iter()
        .map(|node| expand_node(node, slot_contents))
        .collect();

    (expanded, warnings)
}

fn expand_node(node: &PatternNode, slot_contents: &HashMap<String, SlotContent>) -> PatternNode {
    let mut expanded = node.clone();

    // If this is a slot reference, fill it
    if node.kind == "slot" {
        if let Some(slot_ref) = &node.slot_ref {
            if let Some(content) = slot_contents.get(slot_ref) {
                expanded.kind = content.component.clone();
                expanded.props = content.props.clone();
            }
            // If no content, slot remains as empty container
        }
    }

    // Recurse into children
    expanded.children = node
        .children
        .iter()
        .map(|child| expand_node(child, slot_contents))
        .collect();

    expanded
}

// CODEGEN-END
