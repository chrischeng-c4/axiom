---
id: sdd-context-builder-types
fill_sections: [source, overview, schema, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# Context Builder Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/context_builder/types.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ContextEntry` | projects/agentic-workflow/src/context_builder/types.rs | struct | pub | 20 |  |
| `ContextReason` | projects/agentic-workflow/src/context_builder/types.rs | enum | pub | 34 |  |
| `ContextRequest` | projects/agentic-workflow/src/context_builder/types.rs | struct | pub | 47 |  |
| `ContextResponse` | projects/agentic-workflow/src/context_builder/types.rs | struct | pub | 56 |  |
| `ContextStats` | projects/agentic-workflow/src/context_builder/types.rs | struct | pub | 66 |  |
| `ContextTarget` | projects/agentic-workflow/src/context_builder/types.rs | struct | pub | 76 |  |
| `empty` | projects/agentic-workflow/src/context_builder/types.rs | function | pub | 111 | empty() -> Self |
| `parse` | projects/agentic-workflow/src/context_builder/types.rs | function | pub | 88 | parse(input: &str) -> Option<Self> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ContextTarget:
    type: object
    required: [file, symbol]
    description: A single target for context building (file:symbol).
    properties:
      file:
        type: string
      symbol:
        type: string
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ContextRequest:
    type: object
    required: [targets, depth]
    description: Request to build context for one or more targets.
    properties:
      targets:
        type: array
        items:
          $ref: "#/definitions/ContextTarget"
      depth:
        type: integer
        x-rust-type: u32
        x-serde-default: "default_depth"
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ContextReason:
    type: string
    enum: [Target, ImportedByTarget, CalledByTarget, TransitiveDep, CallsTarget, TransitiveCaller, TestFile]
    description: Reason why a file appears in the context.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq, Eq]
      serde_rename_all: snake_case

  ContextEntry:
    type: object
    required: [path, reason, symbols, depth, score]
    description: A single file entry in the context result.
    properties:
      path:
        type: string
      reason:
        $ref: "#/definitions/ContextReason"
      symbols:
        type: array
        items: { type: string }
      depth:
        type: integer
        x-rust-type: u32
      score:
        type: number
        x-rust-type: f64
        x-serde-skip: "serializing"
        description: "Score is internal; never serialized."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ContextStats:
    type: object
    required: [targets_resolved, targets_unresolved, files_scanned, time_ms]
    description: Statistics about the context building process.
    properties:
      targets_resolved:
        type: integer
        x-rust-type: usize
      targets_unresolved:
        type: integer
        x-rust-type: usize
      files_scanned:
        type: integer
        x-rust-type: usize
      time_ms:
        type: integer
        x-rust-type: u64
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ContextResponse:
    type: object
    required: [must_read, may_affect, type_context, stats]
    description: Full context response.
    properties:
      must_read:
        type: array
        items:
          $ref: "#/definitions/ContextEntry"
      may_affect:
        type: array
        items:
          $ref: "#/definitions/ContextEntry"
      type_context:
        type: object
        x-rust-type: "HashMap<String, String>"
      stats:
        $ref: "#/definitions/ContextStats"
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Source
<!-- type: source lang: rust -->

```rust
impl ContextTarget {
    /// Parse a `file:symbol` string into a ContextTarget.
    ///
    /// Returns `None` if the string doesn't contain a `:` separator.
    pub fn parse(input: &str) -> Option<Self> {
        let (file, symbol) = input.rsplit_once(':')?;
        if file.is_empty() || symbol.is_empty() {
            return None;
        }
        Some(Self {
            file: file.to_string(),
            symbol: symbol.to_string(),
        })
    }
}

fn default_depth() -> u32 {
    2
}

// ============================================================================
// Response Types
// ============================================================================

impl ContextResponse {
    /// Create an empty response with zero stats.
    pub fn empty() -> Self {
        Self {
            must_read: Vec::new(),
            may_affect: Vec::new(),
            type_context: HashMap::new(),
            stats: ContextStats {
                targets_resolved: 0,
                targets_unresolved: 0,
                files_scanned: 0,
                time_ms: 0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_target_parse_valid() {
        let t = ContextTarget::parse("src/services/user.py:get_user").unwrap();
        assert_eq!(t.file, "src/services/user.py");
        assert_eq!(t.symbol, "get_user");
    }

    #[test]
    fn test_context_target_parse_colon_in_path() {
        let t = ContextTarget::parse("C:/src/user.py:get_user").unwrap();
        assert_eq!(t.file, "C:/src/user.py");
        assert_eq!(t.symbol, "get_user");
    }

    #[test]
    fn test_context_target_parse_no_colon() {
        assert!(ContextTarget::parse("not_valid_format").is_none());
    }

    #[test]
    fn test_context_target_parse_empty_file() {
        assert!(ContextTarget::parse(":symbol").is_none());
    }

    #[test]
    fn test_context_target_parse_empty_symbol() {
        assert!(ContextTarget::parse("file:").is_none());
    }

    #[test]
    fn test_context_target_parse_empty_string() {
        assert!(ContextTarget::parse("").is_none());
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/context_builder/types.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - ContextTarget
      - ContextRequest
      - ContextReason
      - ContextEntry
      - ContextStats
      - ContextResponse
    description: |
      Codegen replaces all 6 type declarations.
      `ContextEntry.score` is emitted with `#[serde(skip_serializing)]`
      via `x-serde-skip: "serializing"`.
  - path: projects/agentic-workflow/src/context_builder/types.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Regenerate parsing helpers, default-depth logic, response constructors,
      and tests from the source section.
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->
**Verdict:** approved

- [overview] 6-type scope; skip_serializing behavior noted.
- [schema] x-serde-default fn-name + HashMap via x-rust-type proven.
- [changes] codegen + hand-written split correct.
