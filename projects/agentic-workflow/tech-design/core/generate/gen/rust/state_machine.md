---
id: sdd-generate-gen-rust-state-machine
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# State Machine Behavioral Generator

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/gen/rust/state_machine.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `StateMachineGenOutput` | projects/agentic-workflow/src/generate/gen/rust/state_machine.rs | struct | pub | 23 |  |
| `generate_state_machine` | projects/agentic-workflow/src/generate/gen/rust/state_machine.rs | function | pub | 35 | generate_state_machine(     content: &StateMachineContent,     spec_path: &str,     config: &RustConfig, ) -> StateMachineGenOutput |
| `snake_to_pascal` | projects/agentic-workflow/src/generate/gen/rust/state_machine.rs | function | pub | 257 | snake_to_pascal(s: &str) -> String |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/gen/rust/state_machine.rs -->
```rust

//! State machine behavioral generator.
//!
//! Reads `StateMachineContent` parsed from Mermaid Plus frontmatter and generates:
//! - Rust enum with variants for each state (snake_case → PascalCase)
//! - `is_terminal()` bool returning true for terminal states
//! - `is_transient()` bool for transient/choice states
//! - `next()` match skeleton with SPEC-REF markers for routing logic
//!

// @spec projects/agentic-workflow/tech-design/core/logic/codegen-behavioral.md#R1

use crate::generate::diagrams::content::state_machine::{StateKind, StateMachineContent};
use crate::generate::marker::{emit_spec_ref, Lang};
use crate::generate::types::RustConfig;

/// Output from state machine code generation.
#[derive(Debug, Clone)]
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/state_machine.md#source
pub struct StateMachineGenOutput {
    /// The generated Rust code (enum + impls), without CODEGEN markers.
    pub code: String,
    /// List of SPEC-REF markers emitted inside the generated code.
    pub spec_refs: Vec<String>,
}

/// Generate Rust enum + impl from a `StateMachineContent`.
///
/// Returns `StateMachineGenOutput` containing the generated code string
/// and list of emitted SPEC-REF references.
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-behavioral.md#R1
pub fn generate_state_machine(
    content: &StateMachineContent,
    spec_path: &str,
    config: &RustConfig,
) -> StateMachineGenOutput {
    let vis = config.vis_prefix();
    let derive = config.derive_attr();
    let serde_rename = config.serde_rename_attr();

    // Sort state IDs for deterministic output
    let mut state_ids: Vec<&str> = content.nodes.keys().map(|s| s.as_str()).collect();
    state_ids.sort();

    // Emit `#[default]` on the `initial` variant only when Default is in derives.
    let emit_default_attr = config.derives.iter().any(|d| d == "Default");

    // Build enum variants with optional /// doc comments and #[default] attribute.
    let variants: Vec<String> = state_ids
        .iter()
        .map(|id| {
            let variant = snake_to_pascal(id);
            let node = &content.nodes[*id];
            let mut out = String::new();
            if let Some(desc) = node.description.as_deref() {
                for doc_line in desc.lines() {
                    out.push_str(&format!("    /// {}\n", doc_line));
                }
            }
            if emit_default_attr && *id == content.initial {
                out.push_str("    #[default]\n");
            }
            out.push_str(&format!("    {},", variant));
            out
        })
        .collect();

    // Build is_terminal() match arms
    let terminal_ids = content.terminal_ids();
    let terminal_arms: Vec<String> = terminal_ids
        .iter()
        .map(|id| format!("            Self::{} => true,", snake_to_pascal(id)))
        .collect();

    // Build is_transient() match arms
    let transient_ids = content.transient_ids();
    let transient_arms: Vec<String> = transient_ids
        .iter()
        .map(|id| format!("            Self::{} => true,", snake_to_pascal(id)))
        .collect();

    // Build next() match arms with SPEC-REF markers — only when enabled.
    let mut spec_refs = Vec::new();
    let next_arms: Vec<String> = if content.emit_next_fn {
        state_ids
            .iter()
            .map(|id| {
                let variant = snake_to_pascal(id);
                let node = &content.nodes[*id];
                if node.kind == StateKind::Terminal {
                    format!("            Self::{} => None,", variant)
                } else {
                    let marker = emit_spec_ref(spec_path, &content.id, &format!("Implement routing logic for state {}", id), Lang::Rust);
                    spec_refs.push(format!("{}#{}", spec_path, content.id));
                    format!(
                        "            Self::{} => {{\n                {}\n                None\n            }}",
                        variant,
                        marker.replace('\n', "\n                ")
                    )
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    // Resolve the enum name: explicit override > snake_to_pascal(id).
    let type_name = content
        .type_name
        .as_deref()
        .map(|s| s.to_string())
        .unwrap_or_else(|| snake_to_pascal(&content.id));

    // Assemble the generated code
    let mut lines = Vec::new();

    // Enum definition
    if !derive.is_empty() {
        lines.push(derive.clone());
    }
    if !serde_rename.is_empty() {
        lines.push(serde_rename.clone());
    }
    lines.push(format!("{}enum {} {{", vis, type_name));
    for variant in &variants {
        lines.push(variant.clone());
    }
    lines.push("}".to_string());
    lines.push(String::new());

    // impl block
    lines.push(format!("impl {} {{", type_name));
    lines.push(String::new());

    // is_terminal()
    lines.push(format!("    {}fn is_terminal(&self) -> bool {{", vis));
    if terminal_arms.is_empty() {
        lines.push("        false".to_string());
    } else {
        lines.push("        match self {".to_string());
        for arm in &terminal_arms {
            lines.push(arm.clone());
        }
        lines.push("            _ => false,".to_string());
        lines.push("        }".to_string());
    }
    lines.push("    }".to_string());
    lines.push(String::new());

    // is_transient() — only emit when there ARE transient states.
    if !transient_arms.is_empty() {
        lines.push(format!("    {}fn is_transient(&self) -> bool {{", vis));
        lines.push("        match self {".to_string());
        for arm in &transient_arms {
            lines.push(arm.clone());
        }
        lines.push("            _ => false,".to_string());
        lines.push("        }".to_string());
        lines.push("    }".to_string());
        lines.push(String::new());
    }

    // Named classifications → `is_<name>()` methods.
    for (name, ids) in &content.classifications {
        lines.push(format!("    {}fn is_{}(&self) -> bool {{", vis, name));
        if ids.is_empty() {
            lines.push("        false".to_string());
        } else {
            lines.push("        match self {".to_string());
            let mut sorted = ids.clone();
            sorted.sort();
            for id in &sorted {
                lines.push(format!(
                    "            Self::{} => true,",
                    snake_to_pascal(id)
                ));
            }
            lines.push("            _ => false,".to_string());
            lines.push("        }".to_string());
        }
        lines.push("    }".to_string());
        lines.push(String::new());
    }

    // can_transition_to() — emitted from edges. Tuple match over (self, next).
    if !content.edges.is_empty() {
        lines.push(format!(
            "    {}fn can_transition_to(&self, next: Self) -> bool {{",
            vis
        ));
        lines.push("        match (self, next) {".to_string());
        // Sort edges for deterministic output: by from, then to.
        let mut sorted_edges: Vec<&crate::generate::diagrams::content::state_machine::Transition> =
            content.edges.iter().collect();
        sorted_edges.sort_by(|a, b| a.from.cmp(&b.from).then_with(|| a.to.cmp(&b.to)));
        for edge in &sorted_edges {
            lines.push(format!(
                "            (Self::{}, Self::{}) => true,",
                snake_to_pascal(&edge.from),
                snake_to_pascal(&edge.to),
            ));
        }
        lines.push("            _ => false,".to_string());
        lines.push("        }".to_string());
        lines.push("    }".to_string());
        lines.push(String::new());
    }

    // next() — only emit when explicitly requested via emit_next_fn.
    if content.emit_next_fn {
        lines.push(format!(
            "    {}fn next(&self, event: &str) -> Option<Self> {{",
            vis
        ));
        lines.push("        match self {".to_string());
        for arm in &next_arms {
            lines.push(arm.clone());
        }
        lines.push("        }".to_string());
        lines.push("    }".to_string());
    } else {
        // Drop trailing blank line we pushed for spacing after the previous method.
        if lines.last().map(|s| s.is_empty()).unwrap_or(false) {
            lines.pop();
        }
    }
    lines.push("}".to_string());

    // Auto-infer `use` imports from config derives.
    let body = lines.join("\n");
    let mut imports: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    if config
        .derives
        .iter()
        .any(|d| d == "Serialize" || d == "Deserialize")
    {
        imports.insert("use serde::{Deserialize, Serialize};".to_string());
    }

    let code = if imports.is_empty() {
        body
    } else {
        let mut out = imports.into_iter().collect::<Vec<_>>().join("\n");
        out.push_str("\n\n");
        out.push_str(&body);
        out
    };

    StateMachineGenOutput { code, spec_refs }
}

/// Convert snake_case or kebab-case identifier to PascalCase enum variant name.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/state_machine.md#source
pub fn snake_to_pascal(s: &str) -> String {
    s.split(|c| c == '_' || c == '-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::diagrams::content::state_machine::{StateKind, StateNode, Transition};
    use std::collections::HashMap;

    fn make_sm() -> StateMachineContent {
        let mut nodes = HashMap::new();
        nodes.insert(
            "change_inited".to_string(),
            StateNode {
                kind: StateKind::Initial,
                label: None,
                description: None,
            },
        );
        nodes.insert(
            "pre_clarifications_created".to_string(),
            StateNode {
                kind: StateKind::Normal,
                label: None,
                description: None,
            },
        );
        nodes.insert(
            "change_archived".to_string(),
            StateNode {
                kind: StateKind::Terminal,
                label: None,
                description: None,
            },
        );

        StateMachineContent {
            id: "state_phase".to_string(),
            initial: "change_inited".to_string(),
            nodes,
            edges: vec![
                Transition {
                    from: "change_inited".to_string(),
                    to: "pre_clarifications_created".to_string(),
                    event: Some("advance".to_string()),
                    guard: None,
                    label: None,
                },
                Transition {
                    from: "pre_clarifications_created".to_string(),
                    to: "change_archived".to_string(),
                    event: Some("archive".to_string()),
                    guard: None,
                    label: None,
                },
            ],
            title: None,
            type_name: None,
            classifications: Default::default(),
            emit_next_fn: true,
        }
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-behavioral.md#R1
    #[test]
    fn test_generates_enum_variants_from_state_machine() {
        let sm = make_sm();
        let config = crate::generate::types::RustConfig::default();
        let output = generate_state_machine(&sm, "spec.md", &config);

        // Scenario S1: enum variants for all states
        assert!(
            output.code.contains("enum StatePhase"),
            "Should contain enum name"
        );
        assert!(
            output.code.contains("ChangeInited,"),
            "Should have ChangeInited variant"
        );
        assert!(
            output.code.contains("PreClarificationsCreated,"),
            "Should have PreClarificationsCreated variant"
        );
        assert!(
            output.code.contains("ChangeArchived,"),
            "Should have ChangeArchived variant"
        );
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-behavioral.md#R1
    #[test]
    fn test_generates_is_terminal() {
        let sm = make_sm();
        let config = crate::generate::types::RustConfig::default();
        let output = generate_state_machine(&sm, "spec.md", &config);

        // is_terminal() should return true for ChangeArchived
        assert!(
            output.code.contains("fn is_terminal"),
            "Should have is_terminal()"
        );
        assert!(
            output.code.contains("Self::ChangeArchived => true"),
            "ChangeArchived should be terminal"
        );
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-behavioral.md#R1
    #[test]
    fn test_generates_next_skeleton_with_spec_ref() {
        let sm = make_sm();
        let config = crate::generate::types::RustConfig::default();
        let output = generate_state_machine(&sm, "spec.md", &config);

        // Scenario S1: SPEC-REF marker in next() skeleton
        assert!(output.code.contains("fn next"), "Should have next()");
        assert!(
            output.code.contains("SPEC-REF"),
            "Should contain SPEC-REF markers"
        );
        assert!(
            !output.spec_refs.is_empty(),
            "Should emit spec_refs for non-terminal states"
        );
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-behavioral.md#R5
    #[test]
    fn test_terminal_state_returns_none_in_next() {
        let sm = make_sm();
        let config = crate::generate::types::RustConfig::default();
        let output = generate_state_machine(&sm, "spec.md", &config);

        // Terminal states should return None in next() without SPEC-REF
        assert!(
            output.code.contains("Self::ChangeArchived => None"),
            "Terminal state should return None in next()"
        );
    }

    #[test]
    fn test_snake_to_pascal() {
        assert_eq!(snake_to_pascal("change_inited"), "ChangeInited");
        assert_eq!(
            snake_to_pascal("pre_clarifications_created"),
            "PreClarificationsCreated"
        );
        assert_eq!(snake_to_pascal("simple"), "Simple");
    }

    #[test]
    fn test_derives_applied_from_config() {
        let sm = make_sm();
        let config = crate::generate::types::RustConfig::default();
        let output = generate_state_machine(&sm, "spec.md", &config);
        assert!(
            output.code.contains("#[derive("),
            "Should have derive attribute"
        );
        assert!(output.code.contains("Debug"), "Should derive Debug");
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/gen/rust/state_machine.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete state-machine behavioral generator
      module.
```

# Reviews

