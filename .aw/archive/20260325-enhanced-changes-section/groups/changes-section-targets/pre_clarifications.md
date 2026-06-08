---
change: enhanced-changes-section
group: changes-section-targets
date: 2026-03-25
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: How does cclab-lens expose function/struct lookup?
- **Answer**: fillback/ast.rs has AstExtractor::parse_file() that returns ModuleInfo with Vec<Symbol>. Symbol has {name, kind: SymbolKind, signature, line, is_public}. SymbolKind supports Function, Struct, Enum, Interface, Class, Module, Constant, Type. Currently only has start line — need to add end_line to get full source range. Tree-sitter nodes have start_byte/end_byte internally (used by node_text()). This is a Rust API, not CLI-only.

### Q2: General
- **Question**: Does the changes section currently have a dedicated CLI generator?
- **Answer**: The changes section uses the generic artifact CLI (cclab sdd artifact create-change-spec with --type changes). The section_prompts in spec_service.rs define CLI flags per type. Currently changes section has no structured flags — it's prose-only YAML written by the agent. Need to add --targets and --do-not-touch flags.

