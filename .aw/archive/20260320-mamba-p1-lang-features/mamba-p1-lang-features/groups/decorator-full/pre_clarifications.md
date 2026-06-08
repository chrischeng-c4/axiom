---
change: mamba-p1-lang-features
group: decorator-full
date: 2026-03-20
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Current parser decorator limitation
- **Answer**: Current parser supports stacked decorators with simple names. Limitation: no call expressions or attribute access in decorator position.

### Q2: General
- **Question**: Type checker decorator model
- **Answer**: Use Any/unknown fallback for decorator return-type inference. Precise higher-order modeling deferred.

### Q3: General
- **Question**: Special decorator handling
- **Answer**: @property, @staticmethod, @classmethod special-cased with hardcoded semantics in type checker.

### Q4: General
- **Question**: PEP 614 full compliance
- **Answer**: Full PEP 614 — any expression in decorator position, including @buttons[0], @f()(x).

