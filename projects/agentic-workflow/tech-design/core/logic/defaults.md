---
id: projects-sdd-src-defaults-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core logic modules define AW Core defaults, exports, and workflow invariants."
---

# Standardized projects/agentic-workflow/src/defaults.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/defaults.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CLAUDE_BALANCED_MODEL` | projects/agentic-workflow/src/defaults.rs | constant | pub | 22 |  |
| `CLAUDE_DEEP_MODEL` | projects/agentic-workflow/src/defaults.rs | constant | pub | 23 |  |
| `CLAUDE_FAST_MODEL` | projects/agentic-workflow/src/defaults.rs | constant | pub | 21 |  |
| `CODEX_MODEL` | projects/agentic-workflow/src/defaults.rs | constant | pub | 11 |  |
| `CODEX_REASONING_HIGH` | projects/agentic-workflow/src/defaults.rs | constant | pub | 14 |  |
| `CODEX_REASONING_LOW` | projects/agentic-workflow/src/defaults.rs | constant | pub | 12 |  |
| `CODEX_REASONING_MEDIUM` | projects/agentic-workflow/src/defaults.rs | constant | pub | 13 |  |
| `CODEX_REASONING_XHIGH` | projects/agentic-workflow/src/defaults.rs | constant | pub | 15 |  |
| `CODEX_SPARK_MODEL` | projects/agentic-workflow/src/defaults.rs | constant | pub | 18 |  |
| `GEMINI_FLASH_MODEL` | projects/agentic-workflow/src/defaults.rs | constant | pub | 7 |  |
| `GEMINI_PRO_MODEL` | projects/agentic-workflow/src/defaults.rs | constant | pub | 8 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:claim-code -->

<!-- source-snapshot: path=projects/agentic-workflow/src/defaults.rs -->
```rust
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
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/defaults.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:claim-code>"
    description: |
      Source template owns the default model and reasoning-level constants.
```
