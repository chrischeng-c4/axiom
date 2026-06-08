---
id: projects-sdd-src-generate-patterns-resolver-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Standardized projects/agentic-workflow/src/generate/patterns/resolver.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/patterns/resolver.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `expand_pattern` | projects/agentic-workflow/src/generate/patterns/resolver.rs | function | pub | 20 | expand_pattern(     pattern: &UxPattern,     slot_contents: &HashMap<String, SlotContent>, ) -> (Vec<PatternNode>, Vec<String>) |
| `resolve_pattern` | projects/agentic-workflow/src/generate/patterns/resolver.rs | function | pub | 10 | resolve_pattern(pattern_id: &str) -> Option<&'static UxPattern> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/patterns/resolver.rs -->
```rust
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
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/patterns/resolver.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete UX pattern resolver module.
```
