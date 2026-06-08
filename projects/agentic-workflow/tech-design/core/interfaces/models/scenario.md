---
id: sdd-models-scenario
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# Scenario

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/models/scenario.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Scenario` | projects/agentic-workflow/src/models/scenario.rs | struct | pub | 8 |  |
| `new` | projects/agentic-workflow/src/models/scenario.rs | function | pub | 22 | new(name: impl Into<String>) -> Self |
| `validate` | projects/agentic-workflow/src/models/scenario.rs | function | pub | 53 | validate(&self) -> Result<(), String> |
| `with_and` | projects/agentic-workflow/src/models/scenario.rs | function | pub | 44 | with_and(mut self, clause: impl Into<String>) -> Self |
| `with_then` | projects/agentic-workflow/src/models/scenario.rs | function | pub | 38 | with_then(mut self, clause: impl Into<String>) -> Self |
| `with_when` | projects/agentic-workflow/src/models/scenario.rs | function | pub | 32 | with_when(mut self, clause: impl Into<String>) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  Scenario:
    type: object
    required: [name, when, then, and]
    description: BDD-style scenario with WHEN/THEN/AND clause lists.
    properties:
      name:
        type: string
        description: "Scenario name (e.g., \"Valid credentials\")."
      when:
        type: array
        items:
          type: string
        description: "WHEN clauses (preconditions)."
      then:
        type: array
        items:
          type: string
        description: "THEN clauses (expected outcomes)."
      and:
        type: array
        items:
          type: string
        description: "AND clauses (additional conditions, optional)."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq]
    x-constructor:
      name: new
      doc: "Create a new scenario with the given name and empty clause lists."
      impl_mode: codegen
      args:
        - { name: name, rust_type: "impl Into<String>", into: String }
      init:
        when: "Vec::new()"
        then: "Vec::new()"
        and: "Vec::new()"
    x-methods:
      - name: with_when
        returns: Self
        impl_mode: codegen
        doc: "Push a WHEN clause and return self."
        args:
          - { name: clause, rust_type: "impl Into<String>" }
        body: "self.when.push(clause.into()); self"
      - name: with_then
        returns: Self
        impl_mode: codegen
        doc: "Push a THEN clause and return self."
        args:
          - { name: clause, rust_type: "impl Into<String>" }
        body: "self.then.push(clause.into()); self"
      - name: with_and
        returns: Self
        impl_mode: codegen
        doc: "Push an AND clause and return self."
        args:
          - { name: clause, rust_type: "impl Into<String>" }
        body: "self.and.push(clause.into()); self"
      - name: validate
        returns: "Result<(), String>"
        impl_mode: hand-written
        doc: |
          Validate that the scenario has at least one WHEN clause and at least
          one THEN clause. Returns Err with a descriptive message on failure.
```
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/models/scenario.rs -->
```rust
use serde::{Deserialize, Serialize};

/// BDD-style scenario with WHEN/THEN/AND clause lists.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/scenario.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Scenario {
    /// Scenario name (e.g., "Valid credentials").
    pub name: String,
    /// WHEN clauses (preconditions).
    pub when: Vec<String>,
    /// THEN clauses (expected outcomes).
    pub then: Vec<String>,
    /// AND clauses (additional conditions, optional).
    pub and: Vec<String>,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/scenario.md#schema.impls
impl Scenario {
    /// Create a new scenario with the given name and empty clause lists.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            when: Vec::new(),
            then: Vec::new(),
            and: Vec::new(),
        }
    }

    /// Push a WHEN clause and return self.
    pub fn with_when(mut self, clause: impl Into<String>) -> Self {
        self.when.push(clause.into());
        self
    }

    /// Push a THEN clause and return self.
    pub fn with_then(mut self, clause: impl Into<String>) -> Self {
        self.then.push(clause.into());
        self
    }

    /// Push an AND clause and return self.
    pub fn with_and(mut self, clause: impl Into<String>) -> Self {
        self.and.push(clause.into());
        self
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/scenario.md#source
impl Scenario {
    /// Validate that scenario has at least one WHEN and one THEN clause
    pub fn validate(&self) -> Result<(), String> {
        if self.when.is_empty() {
            return Err(format!("Scenario '{}' has no WHEN clauses", self.name));
        }
        if self.then.is_empty() {
            return Err(format!("Scenario '{}' has no THEN clauses", self.name));
        }
        Ok(())
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/models/scenario.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete scenario model module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Clear and accurate: codegen vs hand-written boundary is unambiguously stated; the two-impl-block topology and prepare-commit prerequisite are both called out.
- [schema] `x-constructor`, `x-methods` with inline `body:` literals, and `validate` marked `impl_mode: hand-written` are complete and machine-readable; `x-rust-struct` derive list matches expected serde/debug traits.
- [changes] Two change entries correctly model the codegen and hand-written regions; `replaces: [Scenario, "impl Scenario"]` aligns with the attribute-sweep semantics in `apply.rs`; the gen-code guard ("must not touch it") is explicit.
