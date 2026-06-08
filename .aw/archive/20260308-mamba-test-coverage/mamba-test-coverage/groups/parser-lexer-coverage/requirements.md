---
change: mamba-test-coverage
group: parser-lexer-coverage
date: 2026-03-08
---

# Requirements

Tracking issue #739 covers overall test coverage improvement. This group focuses on Parser (93.7%→95-98%) and Lexer (51%→95-98%) line coverage. Parser gap is small (~88 uncovered lines across parser/mod.rs, type_expr.rs, pattern.rs, expr.rs). Lexer gap is mostly in lexer/token.rs (97 uncovered lines). Add tests for uncovered parser rules, all token types, escape sequences, string prefixes, mixed indentation, Unicode identifiers.
