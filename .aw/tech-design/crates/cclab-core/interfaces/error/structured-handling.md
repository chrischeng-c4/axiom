---
id: structured-error-handling
type: spec
title: "Structured Error Handling"
version: 1
spec_type: algorithm
main_spec_ref: "cclab-core/interfaces/error/structured-handling.md"
fill_sections: [overview, schema, logic, interaction, changes]
design_elements:
  has_mermaid: true
  has_json_schema: true
  diagrams:
    - type: flowchart
      title: "Error Classification Flow"
---

# Structured Error Handling

## Overview
<!-- type: overview lang: markdown -->

cclab-core exposes a structured `DataBridgeError` enum for Rust call sites and
binding-neutral utilities for sanitized host-runtime errors. The Rust core
contract does not depend on native-extension bindings. The current contract
covers database, serialization, validation, connection, query, and
PostgreSQL-specific classification while keeping sensitive connection data out
of production error messages.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  DataBridgeError:
    type: object
    required: [variant, message]
    properties:
      variant:
        type: string
        enum:
          - MongoDB
          - Database
          - Serialization
          - Deserialization
          - Connection
          - Query
          - Validation
          - Internal
          - Conflict
          - ForeignKey
          - Deadlock
          - Timeout
          - Transient
      message:
        type: string

  ErrorCategory:
    type: string
    enum:
      - Connection
      - Authentication
      - Timeout
      - Validation
      - Operation
      - Unknown

  ErrorClassification:
    type: object
    required: [retryable, constraint_violation]
    properties:
      retryable:
        type: array
        items:
          type: string
          enum: [Deadlock, Timeout, Transient]
      constraint_violation:
        type: array
        items:
          type: string
          enum: [Conflict, ForeignKey]
```

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: data-bridge-error-classification
title: DataBridge Error Classification
---
flowchart TD
    Err[DataBridgeError] --> Retry{is_retryable?}
    Retry -->|Deadlock Timeout Transient| Retryable[retry may be attempted]
    Retry -->|other variants| Constraint{is_constraint_violation?}
    Constraint -->|Conflict ForeignKey| ConstraintOut[constraint violation]
    Constraint -->|other variants| Fatal[non-retryable general error]

    Sqlx[sqlx::Error] --> SqlState{SQLSTATE available?}
    SqlState -->|23505 or 23P01| Conflict[Conflict]
    SqlState -->|23503| ForeignKey[ForeignKey]
    SqlState -->|23502 or 23514| Validation[Validation]
    SqlState -->|40P01| Deadlock[Deadlock]
    SqlState -->|40001 or class 40| Transient[Transient]
    SqlState -->|class 08| Connection[Connection]
    SqlState -->|57P01 or 57P02 or 57P03| Transient
    SqlState -->|fallback| Database[Database]
```

## Interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: binding-error-sanitization-flow
title: Binding Error Sanitization Flow
---
flowchart TD
    Input[Display error] --> Categorize[categorize_error]
    Categorize --> Category{ErrorCategory}
    Category -->|Connection| HostConnection[connection error]
    Category -->|Authentication| HostAuth[authentication error]
    Category -->|Timeout| HostTimeout[timeout error]
    Category -->|Validation| HostValidation[validation error]
    Category -->|Operation or Unknown| HostRuntime[runtime error]
    Input --> Sanitize{debug_mode?}
    Sanitize -->|true| Raw[raw message]
    Sanitize -->|false| Redact[redact connection strings credentials IPs auth tokens]
    Redact --> HostErr[host-runtime error message]
    Raw --> HostErr
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: crates/cclab-core/src/error.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: "Maintain DataBridgeError, Result alias, retryability and constraint classification."
  - path: crates/cclab-core/src/error.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Classify serde, MongoDB, and sqlx errors into DataBridgeError variants."
  - path: crates/cclab-core/src/error_utils.rs
    action: modify
    section: interaction
    impl_mode: hand-written
    description: "Sanitize sensitive error text and categorize errors without a native-extension dependency."
  - path: crates/cclab-core/Cargo.toml
    action: modify
    section: interaction
    impl_mode: hand-written
    description: "Keep cclab-core free of native-extension binding dependencies."
  - path: .aw/tech-design/crates/cclab-core/README.md
    action: modify
    section: overview
    impl_mode: hand-written
    description: "Link the normalized structured error handling spec."
```
