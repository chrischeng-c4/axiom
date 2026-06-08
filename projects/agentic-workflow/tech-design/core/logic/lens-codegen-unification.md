---
id: lens-codegen-unification
type: spec
title: "Lens Codegen Unification & Generator Migration"
version: 1
spec_type: utility
spec_group: cclab-lens
merge_strategy: new
created_at: 2026-02-14T11:30:06.396922+00:00
updated_at: 2026-02-14T11:30:06.396922+00:00
requirements:
  total: 6
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
    - R6
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
depends:
  - spec-ir-contract
changes:
  - file: crates/cclab-lens/src/gen/traits.rs
    action: MODIFY
    description: "Updated CodeGenerator trait with name(), can_generate(SpecIR), generate(SpecIR) methods; added TechStack variants"
  - file: crates/cclab-lens/src/gen/registry.rs
    action: CREATE
    description: "GeneratorRegistry for dispatching SpecIR to generators"
  - file: crates/cclab-lens/src/gen/mod.rs
    action: MODIFY
    description: "Added registry submodule and re-exports"
status: partial
history:
  - timestamp: 2026-02-14T11:30:06.396922+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
  - timestamp: 2026-02-14
    agent: "mainthread"
    action: "R2, R5, R6 implemented; R1, R3, R4 deferred (framework migration)"
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "This codegen logic TD supports CB lifecycle generation and regenerable artifact production."
---

<spec>

# Lens Codegen Unification & Generator Migration

## Overview
<!-- type: doc lang: markdown -->

Migrate SDD's framework generators (FastAPI, Express, Axum) to Lens's gen/ module and unify all code generators under a single CodeGenerator trait that consumes SpecIR. This addresses issues #326 (migrate generators), #327 (Generate scope down), and #328 (Lens unify). After migration, SDD only owns spec format/validation/diagrams; Lens owns all code generation.

## Requirements
<!-- type: doc lang: markdown -->

### R1 - Migrate Generate generators to Lens

```yaml
id: R1
priority: high
status: deferred
```

Move FastApiGenerator, ExpressGenerator, and AxumGenerator from crates/cclab-sdd/src/generators/ to crates/cclab-lens/src/gen/framework/. Deferred to a follow-up change.

### R2 - Unify CodeGenerator trait with SpecIR input

```yaml
id: R2
priority: high
status: implemented
```

Updated Lens's CodeGenerator trait with name(), can_generate(&SpecIR), and generate(&SpecIR, &GenContext) methods. Legacy per-type methods preserved for backward compatibility.

### R3 - Framework generator SpecIR adaptation

```yaml
id: R3
priority: high
status: deferred
```

Adapt migrated FastAPI/Express/Axum generators to accept SpecIR::Api variant. Deferred to follow-up change with R1.

### R4 - Remove generators from Generate

```yaml
id: R4
priority: high
status: deferred
```

After migration, remove the generators/ module from cclab-sdd. Deferred to follow-up change with R1.

### R5 - Update Lens MCP tools

```yaml
id: R5
priority: medium
status: implemented
```

TechStack variants added for FastAPI, Express, AxumFramework. lens_generate_from_spec accepts SpecIR routing.

### R6 - Generator registry

```yaml
id: R6
priority: medium
status: implemented
```

GeneratorRegistry implemented with register/find/generate dispatch based on can_generate() results.

</spec>
