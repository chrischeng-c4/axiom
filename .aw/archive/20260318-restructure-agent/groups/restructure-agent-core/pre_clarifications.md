---
change: restructure-agent
group: restructure-agent-core
date: 2026-03-18
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: SpecStore dependency ordering — how to handle missing SpecStore?
- **Answer**: Block. Wait for SpecStore (#901) to be implemented before proceeding with RestructureAgent. SpecStore is a hard dependency, not optional.

### Q2: General
- **Question**: Agent trait compatibility — how does RestructureAgent relate to existing Agent trait?
- **Answer**: Define a new `TypedAgent<I, O>` generic trait. Type-safe, and other future agents (Architect, Coder, Reviewer) can also use it. The existing untyped `Agent` trait remains for backward compatibility.

### Q3: General
- **Question**: System prompt template location
- **Answer**: Use `include_str!` from `crates/cclab-agent/prompts/restructure.md`. Compile-time embedded, easy to maintain as a separate file.

### Q4: General
- **Question**: Output type location for RestructureInput/Output/Question/StructuredIssue
- **Answer**: New module directory `crates/cclab-agent/src/agents/restructure/` with mod.rs + types.rs. Keeps the module clean and organized.

