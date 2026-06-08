---
change_id: mamba-p2
type: gap_spec_knowledge
created_at: 2026-02-22T11:07:50.484630+00:00
updated_at: 2026-02-22T11:07:50.484630+00:00
---

# Gap Analysis: Spec vs Knowledge

## Gaps

1. **No P2 stdlib spec exists** (severity: high)
   - Spec: mamba-stdlib-core only covers sys/os/math/json. mamba-stdlib-p1 covers builtins/time.
   - Knowledge: stdlib-module-pattern is documented but no spec covers the 20+ new P2 stdlib modules.
   - Impact: P2 modules (re, datetime, collections, itertools, functools, pathlib, random, etc.) have no formal spec.

2. **No runtime types P2 spec** (severity: medium)
   - Spec: mamba-runtime-p1 covers set type only.
   - Knowledge: objdata-variant-pattern documented.
   - Impact: frozenset (#410) has no spec beyond the pattern reference.

3. **No codegen P2 spec** (severity: medium)
   - Spec: mamba-codegen-logic covers comprehensions/generators/pattern matching.
   - Impact: dict/list unpacking (#402), except* (#427), __format__ (#425) have no codegen spec.

4. **No OOP P2 spec** (severity: medium)
   - Spec: mamba-oop-model covers inheritance/super/dunder.
   - Impact: __slots__ (#411), enum (#412), dataclasses (#396), __del__ (#426) have no OOP spec.

5. **class.rs split not in any spec** (severity: medium)
   - Knowledge: file limit rule requires split.
   - No spec addresses the architectural change needed.

## No Contradictions

Existing specs and knowledge are consistent. Gaps are all additive (missing coverage for new P2 features).