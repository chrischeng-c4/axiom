---
change: mamba-test-coverage
group: parser-lexer-coverage
date: 2026-03-08
status: answered
---

# Pre-Clarifications

### Q1: parser-negative-tests
- **Answer**: Both. Add negative test fixtures for parse error cases (they test error recovery paths which are currently uncovered) AND positive tests for uncovered branches. Parser is at 93.7% so the gap is small — focus on the specific uncovered lines in parser/mod.rs (78.3%), type_expr.rs (87.2%), and pattern.rs (89.3%).

### Q2: lexer-token-coverage
- **Answer**: Focus on runtime-reachable paths first. lexer/token.rs likely has Display/Debug impls and token variant constructors that are uncovered. Test all token variants that the parser actually consumes. For Display impls, a simple round-trip test (format each variant, verify output) covers them efficiently.

