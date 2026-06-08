---
change: mamba-type-3arg
group: mamba-type-3arg-core
date: 2026-04-10
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Should the `class` statement and 3-arg `type()` share a single internal class-building primitive?
- **Answer**: Yes. The `class` statement should desugar to or share the same internal `build_class` primitive that 3-arg `type()` calls. This is R7 and is a must-reconcile item if the paths currently diverge.

### Q2: General
- **Question**: What is the priority ordering for requirements?
- **Answer**: R1-R5 are must-have. R6 (metaclass conflict TypeError) and R7 (class statement reconciliation) are should. R8 (__class_cell__ / super() closures) is nice-to-have.

### Q3: General
- **Question**: How should dynamic classes interact with Mamba's type system?
- **Answer**: Classes created by 3-arg type() should be typed as `type` at type-check time. Attribute access on the result is resolved at runtime. The call must not be rejected just because the namespace dict is not statically analyzable.

### Q4: General
- **Question**: What happens on MRO inconsistency?
- **Answer**: MRO failures (inconsistent C3 linearization) must raise TypeError at class-creation time, not produce a broken class object.

### Q5: General
- **Question**: Are custom metaclasses (__new__ / __init__ overrides) in scope?
- **Answer**: No. Custom metaclass __new__/__init__ overrides are out of scope. Only metaclass conflict detection (R6) is in scope.
