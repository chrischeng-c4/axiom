# #1611 — classmethod/staticmethod `.__get__` walls fail to wall

Status: OPEN (p2). Design for implementation.

## Mechanism

4 `type/` dimension guard fixtures expect the checker to REJECT wrong-typed
arguments to `classmethod.__get__`/`staticmethod.__get__`, but the calls pass
through unchecked. Pre-existing (confirmed on pure f15a81220 rebuild during
#1595) — the descriptor wrapper types simply have no signature entry on the
`.__get__` path, so the arg checker has nothing to check against.

## Invariant

This is the NEGATIVE contract domain: `type/` fixtures must be rejected.
Adding the signatures must not over-wall legal descriptor use —
`SomeClass.method` attribute access and normal bound-call paths never route
through an explicit `.__get__` call and must stay untouched.

## Fix direction

Add explicit-`.__get__` signatures for classmethod/staticmethod (receiver +
`(instance, owner)` params) wherever builtin-method signatures live for
walls (curated table, NOT the generated typeshed file). Wall only the
explicit-call shape.

## Verification contract

The 4 guard fixtures flip to correctly-rejected (names in #1595's evidence
comment); `behavior/core/descr` + decorator sweeps show zero new compile
rejects (no over-walling); full gate must not worsen.
