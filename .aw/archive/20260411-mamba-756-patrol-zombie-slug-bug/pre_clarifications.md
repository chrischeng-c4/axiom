---
change: mamba-756-patrol
date: 2026-04-11
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Scope confirmation — which items from the #756 checklist should this change cover?
- **Answer**: Full checklist: (1) sync yield/yield from/send/throw/close/StopIteration.value semantics, (2) iterator protocol __iter__/__next__/StopIteration, (3) async generators (async for, async yield, async iterator protocol), (4) itertools-compatible iteration behavior. All 4 buckets are in scope for this single change.

### Q2: General
- **Question**: Work type — tests only, tests + fix, or fix only?
- **Answer**: Tests + fix non-conformant behavior. Port CPython 3.12 generator/iterator tests as conformance fixtures, run them against mamba, and fix anything that diverges in the same change. Larger scope but keeps test + implementation aligned.

### Q3: General
- **Question**: Test source — port CPython 3.12 test suite, write fresh, or both?
- **Answer**: Port projects/cpython/Lib/test/test_generators.py as pytest fixtures driven through the mamba runtime. Highest-fidelity conformance signal. Augment with mamba-specific regression tests only if a CPython test doesn't cover a mamba-specific path (e.g. Cranelift codegen edges).

