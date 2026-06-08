---
change: mamba-conformance-p1
group: mamba-p1-oop
date: 2026-03-24
---

# Requirements

Fix 5 P1 OOP conformance gaps:

1. **@classmethod** — Cranelift codegen emits wrong parameter count for classmethod. cls should be first param.

2. **@property** — property decorator codegen crashes. Need to implement descriptor protocol dispatch.

3. **getattr/setattr/delattr** — Cranelift IR verifier rejects generated instructions. Need valid IR for attribute reflection.

4. **super().method() return** — super() method calls work for side effects but return values are lost. Need to propagate return from MRO-dispatched call.

5. **Multiple inheritance MRO** — C3 linearization produces wrong order. Need to fix MRO computation in runtime/class.rs.
