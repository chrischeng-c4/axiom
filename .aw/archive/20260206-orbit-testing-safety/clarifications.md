---
change: orbit-testing-safety
date: 2026-02-06
---

# Clarifications

## Q1: Scope
- **Question**: What testing tools to use?
- **Answer**: cargo-fuzz for fuzz testing, Miri for undefined behavior detection
- **Rationale**: Industry-standard tools for Rust safety testing

## Q2: Fuzz Targets
- **Question**: Which components to fuzz?
- **Answer**: Timer wheel, task queue, protocol parsing
- **Rationale**: Core components with complex state machines

## Q3: CI Integration
- **Question**: How to integrate with CI?
- **Answer**: Miri on PRs, fuzz testing periodically
- **Rationale**: Miri is fast enough for PR checks, fuzzing needs longer runs

