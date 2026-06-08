---
change: codegen-td-to-code
date: 2026-04-10
status: answered
---

# Pre-Clarifications

### Q1: Implementation scope
- **Question**: Which crates and files will this change affect?
- **Answer**: crate:sdd — crates/sdd/src/generate/ (extend existing), projects/score/cli/src/commands.rs (new GenCommands variants)
- **Rationale**: Issue scopes explicitly to sdd crate and score CLI entry points

### Q2: Diagram Content types vs Graph<N,E>
- **Question**: Should we use a universal Graph<N,E> or per-diagram Content types?
- **Answer**: Per-diagram Content types (D3). Common envelope = {id, title, content, metadata}. Each diagram has its own Content shape. Not a universal Graph<N,E>.
- **Rationale**: D3 from issue: requirement diagram has 3 collections, flowchart has nodes/edges, sequence has actors/messages. Per-type is cleaner.

### Q3: CODEGEN marker format
- **Question**: What format for CODEGEN block markers?
- **Answer**: // CODEGEN-BEGIN and // CODEGEN-END. Codegen updates content inside, wrapper preserved. Multiple blocks per file allowed (one per spec section). SPEC-MANAGED comment above each block.
- **Rationale**: D4 from issue: no merge conflicts, hand-written code outside preserved.

### Q4: Language target for MVP
- **Question**: Which target languages for MVP?
- **Answer**: Rust only (D2). Abstract type system designed for multi-language but generators target Rust first. Python/TS deferred.
- **Rationale**: D2 from issue: focus on Rust codebase first.

### Q5: Entry points
- **Question**: What CLI entry points?
- **Answer**: score gen diff / apply / render / validate. All 4 integrated into check-alignment. Diff shows drift, apply writes, render regenerates Mermaid body from frontmatter, validate checks frontmatter schema.
- **Rationale**: D16 from issue.

### Q6: XState schema replacement
- **Question**: Should we replace existing XState schemas?
- **Answer**: Yes (D8). Existing *_plus/schema.rs are unused XState complexity. Replace with new Graph-based schemas that match codegen targets directly.
- **Rationale**: D8 from issue: clean up unused complexity.

### Q7: Config layering
- **Question**: How does config layering work?
- **Answer**: 3 levels: global (.score/config.toml [codegen.*]) -> per-spec (x-rust: extensions in section frontmatter) -> per-file (CODEGEN markers scope generated region).
- **Rationale**: D7 from issue.

