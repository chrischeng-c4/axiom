---
number: 758
title: "Py3.12 conformance: Builtins (108 tests → full verification)"
state: open
labels: [enhancement, P0, crate:mamba]
group: "py312-conformance"
---

# #758 — Py3.12 conformance: Builtins (108 tests → full verification)

## Parent

Part of #750

## Goal

Verify all Python builtins match CPython 3.12 behavior. 108 tests exist but need conformance verification (currently assert mamba behavior without comparing to CPython).

## Scope

### Numeric
- [ ] int(), float(), complex(), round(), abs(), pow(), divmod()

### Sequence
- [ ] len(), range(), sorted(), reversed(), enumerate(), zip(), map(), filter()

### String
- [ ] str(), repr(), format(), chr(), ord(), ascii()

### Type
- [ ] type(), isinstance(), issubclass(), callable()
- [ ] hasattr(), getattr(), setattr(), delattr()

### I/O
- [ ] print(), input(), open()

### Other
- [ ] id(), hash(), vars(), dir(), globals(), locals()
- [ ] all(), any(), min(), max(), sum()
- [ ] iter(), next(), slice(), super()
- [ ] exec(), eval(), compile()
- [ ] __import__(), staticmethod(), classmethod()

## Current State

108 builtin tests exist. Need to verify each matches CPython 3.12 edge cases.
