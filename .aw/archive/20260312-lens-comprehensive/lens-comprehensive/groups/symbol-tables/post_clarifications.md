---
change: lens-comprehensive
group: symbol-tables
date: 2026-03-12
status: clarified
---

# Post-Clarifications

## Questions

### Q1: JS reuse TS extractor
- **Question**: Does rust-symbol-analysis spec have a pattern for language aliasing?
- **Answer**: No explicit alias, but the SymbolBuilder trait allows mapping multiple languages to the same builder. JS will delegate to TS builder with a Language::JavaScript wrapper.

