---
id: sdd-models-requirement
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# Requirement

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/models/requirement.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Requirement` | projects/agentic-workflow/src/models/requirement.rs | struct | pub | 10 |  |
| `RequirementDelta` | projects/agentic-workflow/src/models/requirement.rs | enum | pub | 42 |  |
| `added` | projects/agentic-workflow/src/models/requirement.rs | function | pub | 60 | added(self, req: Requirement) -> Self |
| `modified` | projects/agentic-workflow/src/models/requirement.rs | function | pub | 65 | modified(self, req: Requirement) -> Self |
| `new` | projects/agentic-workflow/src/models/requirement.rs | function | pub | 22 | new(name: impl Into<String>, description: impl Into<String>) -> Self |
| `removed` | projects/agentic-workflow/src/models/requirement.rs | function | pub | 70 | removed(self, name: impl Into<String>, reason: impl Into<String>) -> Self |
| `renamed` | projects/agentic-workflow/src/models/requirement.rs | function | pub | 79 | renamed(self, from: impl Into<String>, to: impl Into<String>) -> Self |
| `validate` | projects/agentic-workflow/src/models/requirement.rs | function | pub | 90 | validate(&self) -> Result<(), String> |
| `with_scenario` | projects/agentic-workflow/src/models/requirement.rs | function | pub | 31 | with_scenario(mut self, scenario: Scenario) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  Requirement:
    type: object
    required: [name, description, scenarios]
    description: A requirement with a name, description, and list of validating scenarios.
    properties:
      name:
        type: string
        description: "Requirement name (e.g., \"User Authentication\")."
      description:
        type: string
        description: "Description of the requirement."
      scenarios:
        type: array
        items:
          $ref: "#/definitions/Scenario"
        description: "List of scenarios that validate this requirement."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq]
    x-constructor:
      name: new
      doc: "Create a new requirement with the given name and description, and an empty scenarios list."
      impl_mode: codegen
      args:
        - { name: name, rust_type: "impl Into<String>", into: String }
        - { name: description, rust_type: "impl Into<String>", into: String }
      init:
        scenarios: "Vec::new()"
    x-methods:
      - name: with_scenario
        returns: Self
        impl_mode: codegen
        doc: "Push a scenario and return self."
        args:
          - { name: scenario, rust_type: Scenario }
        body: "self.scenarios.push(scenario); self"
      - name: validate
        returns: "Result<(), String>"
        impl_mode: hand-written
        doc: |
          Validate requirement format and completeness. Checks name and
          description are non-empty and at least one scenario exists;
          also recursively validates each scenario.

  RequirementDelta:
    type: object
    description: |
      Different types of requirement changes for tracking spec evolution.
      Variants carry payload — Added/Modified wrap a full Requirement;
      Removed/Renamed carry named fields.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq]
      variants:
        - name: Added
          kind: tuple
          fields:
            - { rust_type: Requirement }
          doc: "New requirement added."
        - name: Modified
          kind: tuple
          fields:
            - { rust_type: Requirement }
          doc: "Existing requirement modified (full text required)."
        - name: Removed
          kind: struct
          fields:
            - { name: name,      rust_type: String }
            - { name: reason,    rust_type: String }
            - { name: migration, rust_type: "Option<String>" }
          doc: "Requirement removed (name + reason)."
        - name: Renamed
          kind: struct
          fields:
            - { name: from, rust_type: String }
            - { name: to,   rust_type: String }
          doc: "Requirement renamed."
    x-methods:
      - name: added
        returns: Self
        impl_mode: codegen
        doc: "Construct an Added variant."
        args:
          - { name: req, rust_type: Requirement }
        body: "Self::Added(req)"
      - name: modified
        returns: Self
        impl_mode: codegen
        doc: "Construct a Modified variant."
        args:
          - { name: req, rust_type: Requirement }
        body: "Self::Modified(req)"
      - name: removed
        returns: Self
        impl_mode: codegen
        doc: "Construct a Removed variant with a reason; migration defaults to None."
        args:
          - { name: name,   rust_type: "impl Into<String>" }
          - { name: reason, rust_type: "impl Into<String>" }
        body: "Self::Removed { name: name.into(), reason: reason.into(), migration: None }"
      - name: renamed
        returns: Self
        impl_mode: codegen
        doc: "Construct a Renamed variant."
        args:
          - { name: from, rust_type: "impl Into<String>" }
          - { name: to,   rust_type: "impl Into<String>" }
        body: "Self::Renamed { from: from.into(), to: to.into() }"
```
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/models/requirement.rs -->
```rust
use super::Scenario;

use serde::{Deserialize, Serialize};

/// A requirement with a name, description, and list of validating scenarios.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/requirement.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Requirement {
    /// Requirement name (e.g., "User Authentication").
    pub name: String,
    /// Description of the requirement.
    pub description: String,
    /// List of scenarios that validate this requirement.
    pub scenarios: Vec<Scenario>,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/requirement.md#schema.impls
impl Requirement {
    /// Create a new requirement with the given name and description, and an empty scenarios list.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            scenarios: Vec::new(),
        }
    }

    /// Push a scenario and return self.
    pub fn with_scenario(mut self, scenario: Scenario) -> Self {
        self.scenarios.push(scenario);
        self
    }
}

/// Different types of requirement changes for tracking spec evolution.
/// Variants carry payload — Added/Modified wrap a full Requirement;
/// Removed/Renamed carry named fields.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/requirement.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RequirementDelta {
    /// New requirement added.
    Added(Requirement),
    /// Existing requirement modified (full text required).
    Modified(Requirement),
    /// Requirement removed (name + reason).
    Removed {
        name: String,
        reason: String,
        migration: Option<String>,
    },
    /// Requirement renamed.
    Renamed { from: String, to: String },
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/requirement.md#schema.impls
impl RequirementDelta {
    /// Construct an Added variant.
    pub fn added(self, req: Requirement) -> Self {
        Self::Added(req)
    }

    /// Construct a Modified variant.
    pub fn modified(self, req: Requirement) -> Self {
        Self::Modified(req)
    }

    /// Construct a Removed variant with a reason; migration defaults to None.
    pub fn removed(self, name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::Removed {
            name: name.into(),
            reason: reason.into(),
            migration: None,
        }
    }

    /// Construct a Renamed variant.
    pub fn renamed(self, from: impl Into<String>, to: impl Into<String>) -> Self {
        Self::Renamed {
            from: from.into(),
            to: to.into(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/requirement.md#source
impl Requirement {
    /// Validate requirement format and completeness
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Requirement name cannot be empty".to_string());
        }
        if self.description.is_empty() {
            return Err(format!("Requirement '{}' has no description", self.name));
        }
        if self.scenarios.is_empty() {
            return Err(format!("Requirement '{}' has no scenarios", self.name));
        }

        // Validate all scenarios
        for scenario in &self.scenarios {
            scenario.validate()?;
        }

        Ok(())
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/models/requirement.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete requirement model module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Clear and accurate. The mixed-impl-block topology and the first-use-of-method-level-replaces context are well explained.
- [schema] Correct. All three fields present with proper types; `x-constructor`, `x-methods` entries match R2; `impl_mode` split between `codegen` and `hand-written` is explicit; `body:` literal for `with_scenario` follows the scenario-spec pattern.
- [changes] Correct. Method-level `replaces:` entries (`"impl Requirement::new"`, `"impl Requirement::with_scenario"`) satisfy R3; two-entry structure separating codegen and hand-written regions is unambiguous; `RequirementDelta` deferral is explicitly noted in the hand-written entry per R5.

## Review 2
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [schema] `RequirementDelta` definition is correct and complete: `x-rust-enum.variants` uses `kind: tuple` for `Added`/`Modified` (each wrapping `Requirement`) and `kind: struct` for `Removed` (name, reason, migration: Option<String>) and `Renamed` (from, to) — satisfies R1–R4. All four derive attributes match `Requirement`.
- [schema] `x-methods` block on `RequirementDelta` defines all four constructors with `impl_mode: codegen` and `body: literal` expressions — satisfies R5. Body expressions are syntactically correct Rust: `Self::Added(req)`, `Self::Modified(req)`, struct-init for `Removed`/`Renamed` with `.into()` conversions.
- [changes] `replaces:` in the codegen entry now includes `RequirementDelta` and `"impl RequirementDelta"` — satisfies R7. The hand-written entry updated to note RequirementDelta is now codegen (was deferred), removing the deferred-block description. No inconsistency between schema and changes sections.
