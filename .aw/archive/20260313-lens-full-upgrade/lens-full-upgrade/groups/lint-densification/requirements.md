---
change: lens-full-upgrade
group: lint-densification
date: 2026-03-13
---

# Requirements

Expand lint rules for TypeScript (5→15+), JavaScript (5→15+), Rust (5→15+), CSS (5→15+). Cover common patterns: TS — no-unused-vars, no-explicit-any, prefer-const, no-non-null-assertion, no-floating-promises, strict-boolean-expressions, etc. JS — no-eval, no-implied-eval, no-proto, no-with, eqeqeq, no-alert, no-debugger, etc. Rust — unwrap usage, todo/unimplemented, missing error handling, unsafe patterns, clippy-like rules. CSS — no !important, no universal selector, no ID selectors, shorthand properties, z-index management.
