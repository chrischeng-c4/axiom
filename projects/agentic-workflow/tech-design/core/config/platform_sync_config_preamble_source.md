---
id: sdd-config-platform-sync-config-preamble-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Config and platform TDs define AW Core client boundary behavior."
---

# Platform Sync Config Preamble Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/services/platform_sync/config.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AuthConfig` | projects/agentic-workflow/src/services/platform_sync/config.rs | struct | pub | 19 |  |
| `LabelConfig` | projects/agentic-workflow/src/services/platform_sync/config.rs | struct | pub | 29 |  |
| `PlatformConfig` | projects/agentic-workflow/src/services/platform_sync/config.rs | struct | pub | 48 |  |
| `ScopeAutoDetect` | projects/agentic-workflow/src/services/platform_sync/config.rs | struct | pub | 77 |  |
| `ScopeConfig` | projects/agentic-workflow/src/services/platform_sync/config.rs | struct | pub | 85 |  |
| `StatusLabels` | projects/agentic-workflow/src/services/platform_sync/config.rs | struct | pub | 98 |  |
| `TitleConfig` | projects/agentic-workflow/src/services/platform_sync/config.rs | struct | pub | 114 |  |
| `extract_scope_labels` | projects/agentic-workflow/src/services/platform_sync/config.rs | function | pub | 258 | extract_scope_labels(&self, affected_code: &[String]) -> Vec<String> |
| `format_proposal_title` | projects/agentic-workflow/src/services/platform_sync/config.rs | function | pub | 234 | format_proposal_title(&self, change_id: &str, title: &str) -> String |
| `format_spec_title` | projects/agentic-workflow/src/services/platform_sync/config.rs | function | pub | 246 | format_spec_title(&self, change_id: &str, spec_id: &str) -> String |
| `get_token` | projects/agentic-workflow/src/services/platform_sync/config.rs | function | pub | 194 | get_token(&self, project_root: &Path) -> Result<Option<String>> |
| `load` | projects/agentic-workflow/src/services/platform_sync/config.rs | function | pub | 127 | load(project_root: &Path) -> Result<Self> |
| `proposal_label` | projects/agentic-workflow/src/services/platform_sync/config.rs | function | pub | 224 | proposal_label(&self) -> Option<&str> |
| `spec_label` | projects/agentic-workflow/src/services/platform_sync/config.rs | function | pub | 229 | spec_label(&self) -> Option<&str> |
## Source
<!-- type: source lang: rust -->

```rust
//! Platform configuration

use crate::models::change::{RepoPlatformConfig, TechDesignPlatformConfig};
use crate::Result;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/platform_sync/config.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-preamble>"
      - "<handwrite-gap:platform-sync-config-preamble>"
    description: "Source template owns platform sync config docs and non-serde imports."
```
