<!-- HANDWRITE-BEGIN gap="missing-generator:source:09ae5a1d" tracker="pending-tracker" reason="Add semantic coverage for OTLP contract verification tests." -->
---
id: libs-service-http-tests-otlp-tracing-rs
summary: Lossless rust-source-unit coverage for `libs/service-http/tests/otlp_tracing.rs`.
capability_refs:
  - id: shared-http-service-scaffold
    role: primary
    claim: shared-http-service-scaffold-contract
    coverage: full
fill_sections: [overview, source, changes]
---

# Standardized libs/service-http/tests/otlp_tracing.rs

## Overview
<!-- type: overview lang: markdown -->

Contract tests for optional OTLP tracing. They prove that logging-only startup
remains available, resource identity is stable, exporter construction failures
are non-fatal, and the shared request middleware honors W3C trace context.

### Symbols

| Name | Target | Kind | Visibility | Signature |
|---|---|---|---|---|
| `logging_only_default_requires_no_exporter` | `libs/service-http/tests/otlp_tracing.rs` | test | private | `fn logging_only_default_requires_no_exporter()` |
| `otlp_identity_contract_is_stable` | `libs/service-http/tests/otlp_tracing.rs` | test | private | `fn otlp_identity_contract_is_stable()` |
| `exporter_setup_failure_keeps_logging_available` | `libs/service-http/tests/otlp_tracing.rs` | test | private | `fn exporter_setup_failure_keeps_logging_available()` |
| `trace_layer_propagates_w3c_parent_context` | `libs/service-http/tests/otlp_tracing.rs` | test | private | `fn trace_layer_propagates_w3c_parent_context()` |

## Source
<!-- type: rust-source-unit lang: rust -->

```rust
// OTLP contract tests use only local deterministic configuration and header
// fixtures. They do not require a vendor collector or retain bearer values.
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/service-http/tests/otlp_tracing.rs"
    action: create
    section: rust-source-unit
    impl_mode: hand-written
    description: "Verifies optional OTLP initialization and W3C propagation."
```
<!-- HANDWRITE-END -->
