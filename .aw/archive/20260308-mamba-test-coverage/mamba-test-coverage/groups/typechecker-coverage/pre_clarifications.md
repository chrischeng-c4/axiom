---
change: mamba-test-coverage
group: typechecker-coverage
date: 2026-03-08
status: answered
---

# Pre-Clarifications

### Q1: type-error-coverage
- **Answer**: Both. Tests should verify correct type inference results AND cover diagnostic error messages and error recovery paths. Error diagnostics are part of the user-facing contract — incorrect or missing error messages are bugs. Use typecheck fixture files with # EXPECT-ERROR directives to test error paths.

### Q2: protocol-generic-priority
- **Answer**: Both protocols and generics are actively used features. Protocols implement Python's structural subtyping (PEP 544) and generics implement parameterized types. Both are core to the type system and must be thoroughly tested. Priority: generics first (more widely used), then protocols.

