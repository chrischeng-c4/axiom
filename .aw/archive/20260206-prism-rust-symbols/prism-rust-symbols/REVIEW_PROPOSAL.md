# Proposal Review (Iteration 1)

**Change ID**: prism-rust-symbols

## Summary

Proposal is directionally clear, but impact and scope are under-specified for adding a new language backend. Affected areas and feasibility details are incomplete.

## Issues

- **[MEDIUM]** Impact analysis understates scope ("affected_files: ~1") and lists only `crates/cclab-prism/src/semantic/symbols.rs`, but adding Rust symbol extraction typically touches language-specific parsing, tree-sitter integration, tests/fixtures, and possibly schema/interfaces used by hover/definition/ref features.
  - *Recommendation*: Expand impact to include any Rust parser/grammar integration, symbol model changes, tests/fixtures, and downstream consumers (hover/definition/ref). Update affected_files estimate accordingly.
- **[MEDIUM]** Feasibility details are thin: the proposal mentions an AST visitor and type signature parsing but does not specify the Rust AST source (e.g., tree-sitter Rust) or where in Prism the Rust parse tree is obtained.
  - *Recommendation*: Clarify which Rust AST source is used and where the visitor hooks into existing Prism parsing pipeline.
- **[LOW]** Affected specs list includes `rust-symbol-analysis` at `specs/rust-symbol-analysis.md` but there is no rationale for why this spec is the only one impacted or whether it already exists.
  - *Recommendation*: Confirm the spec exists and list any other impacted specs (e.g., symbol model or hover/definition/ref features), or explain why none are affected.

## Verdict

- [ ] APPROVED - Proposal is clear, complete, and ready for spec creation
- [x] NEEDS_REVISION - Has issues that need fixing
- [ ] REJECTED - Fundamental problems with the proposal

**Next Steps**: Revise proposal to expand impact scope and clarify the Rust parsing approach and affected specs, then resubmit for review.
