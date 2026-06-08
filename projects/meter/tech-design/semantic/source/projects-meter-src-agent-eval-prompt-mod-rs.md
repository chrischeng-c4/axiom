---
id: projects-meter-src-agent-eval-prompt-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: json-default-report-envelope-and-findings
    claim: json-default-report-envelope-and-findings
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/agent_eval/prompt/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/agent_eval/prompt/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `engine` | projects/meter/src/agent_eval/prompt/mod.rs | module | pub | 5 |  |
| `registry` | projects/meter/src/agent_eval/prompt/mod.rs | module | pub | 6 |  |
| `template` | projects/meter/src/agent_eval/prompt/mod.rs | module | pub | 7 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/meter/src/agent_eval/prompt/mod.rs -->
````rust
//! Prompt template system for agent evaluation

pub mod engine;
pub mod registry;
pub mod template;

pub use engine::PromptEngine;
pub use registry::PromptRegistry;
pub use template::{FewShotExample, PromptContext, PromptSection, PromptTemplate, PromptVariable};
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/agent_eval/prompt/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/agent_eval/prompt/mod.rs` captured during meter full-codegen standardization.
```
