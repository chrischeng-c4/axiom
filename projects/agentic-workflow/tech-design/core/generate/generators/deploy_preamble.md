---
id: sdd-generate-generators-deploy-preamble
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# DeployGenerator Preamble Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/generators/deploy.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `DeployGenerator` | projects/agentic-workflow/src/generate/generators/deploy.rs | struct | pub | 30 |  |
| `new` | projects/agentic-workflow/src/generate/generators/deploy.rs | function | pub | 38 | new() -> Self |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap generate-generators-deploy-preamble -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/generators/deploy.rs -->
```rust

//! Kubernetes Deployment + Service manifest generator
//!
//! Generates Kubernetes manifests from a [`DeploySpec`] (deploy section type):
//!
//! | Output file        | Description                                    |
//! |--------------------|------------------------------------------------|
//! | `deployment.yaml`  | `apps/v1 Deployment` resource                  |
//! | `service.yaml`     | `v1 Service` (ClusterIP) resource              |
//!
//! The generator implements [`SpecIRGenerator`] and only accepts
//! [`SpecIR::Deploy`] variants.

use super::common::{
    GeneratedFile, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy, SpecIRGenerator,
};
use crate::generate::engine::TemplateEngine;
use crate::generate::spec_ir::{DeploySpec, EnvVar, SpecIR};
use serde::Serialize;

// ---------------------------------------------------------------------------
// DeployGenerator
// ---------------------------------------------------------------------------
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/generators/deploy.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:generate-generators-deploy-preamble>"
    description: "Source template owns module docs, imports, and the generator section header."
```
