# #1557 — exception-subclass `__init__`: three broken shapes

Status: OPEN (p1). Design for implementation.

## Mechanism (probed 2026-07-13, discrimination matrix on the issue)

- P1 `Exception.__init__(self, a)` UNBOUND chain inside a subclass `__init__`,
  then `self.tag = 1`: instance ends up WITHOUT `tag` (no exception raised).
  `super().__init__(a)` (P2) works — the defect is specific to the unbound
  class-attribute call form. Diagnosis step required: determine whether the
  unbound call rebinds/replaces the receiver, or aborts the caller's remaining
  body (compare MIR via --emit mir for P1 vs P2), or the attr store lands on a
  different object.
- P3 no-chain: user `__init__` never calls base → CPython still has
  `str(c) == 'x'` because `BaseException.__new__` pre-stores ctor args; mamba
  renders `<C instance>` (args not pre-stored + str() falls back to generic
  repr instead of the exception str machinery).
- Composite crash: multiple exception subclasses + inline construction in call
  args → `TypeError: 'NoneType' object is not callable` (same message family
  as #1550 singletons / #1582 dict.__or__ — suspected shared
  unbound-dispatch-to-None root; DIAGNOSE TOGETHER before fixing separately).

## Invariant

CPython semantics: `BaseException.__new__` stores `args` at allocation;
`Class.__init__(self, …)` unbound operates on the SAME `self` the caller
holds; statements after the chain call always execute.

## Red lines

#227's `exc.__init__()` bound synthesis and #228's `super().__new__` narrow
arm must stay green (`exception-construction-contracts.md`,
`super-dispatch-and-error-semantics.md`). No blanket changes to
`SUPER_MISSING_INIT_METHOD` fallback.

## Verification contract

Probe matrix P1-P5 (scratchpad probe-exc-init-shape.py / issue #1557 body)
byte-identical vs python3.12; composite 4-class probe no longer crashes;
`_regression/core/exceptions` + `behavior/core/exceptions` focused sweep no
regressions (#227's 17 cited stay green). Cross-check #1550/#1582 fixtures —
if the shared root cause falls out, note gate delta on all three issues.
