---
change: section-type-coverage
group: new-section-types
date: 2026-03-24
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Should all 16 types be added in this change?
- **Answer**: Yes, add all 16 types. This is a spec-only change — adding to the type table, section rules, fill order, and CLI flags is low-risk and additive.

### Q2: General
- **Question**: For grpc and graphql — native IDL or JSON Schema?
- **Answer**: Use JSON Schema, consistent with the existing pattern. rest-api uses OpenAPI (wraps JSON Schema), rpc-api uses OpenRPC (wraps JSON Schema), async-api uses AsyncAPI (wraps JSON Schema). For grpc: JSON envelope with services/methods/streaming + JSON Schema $defs for types → generator outputs .proto files. For graphql: JSON envelope with queries/mutations/subscriptions + JSON Schema $defs → generator outputs .graphql SDL.

### Q3: General
- **Question**: For prompt type — keep as logic or dedicated format?
- **Answer**: Create a dedicated markdown-based format. Prompts have variables, system instructions, few-shot examples — semantically different from flowchart logic.

