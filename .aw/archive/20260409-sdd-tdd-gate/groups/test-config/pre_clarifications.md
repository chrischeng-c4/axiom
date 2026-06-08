---
change: sdd-tdd-gate
group: test-config
date: 2026-04-08
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Should TestConfig support a global test_cmd default that scopes inherit if they don't specify their own?
- **Answer**: Yes, TestConfig should have an optional global `test_cmd` default. TestScope entries that omit `test_cmd` inherit from the global default. This avoids repetition when most scopes use the same test runner (e.g. cargo test).

### Q2: General
- **Question**: Should the changes patterns use gitignore-style globs (like GitLab CI) or standard Unix globs?
- **Answer**: Use gitignore-style globs for consistency with GitLab CI `changes:` semantics. The `globset` crate with gitignore-compatible matching is the right choice.

