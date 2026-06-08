"""Hot-loop bench for `collections.abc.Iterable` /
`collections.abc.Hashable` / `collections.abc.Sized` /
`collections.abc.Container` module-attribute reads (#1450).

End-user scenario: hot type-classification code that
introspects the abstract-base-class family on every value —
e.g. `isinstance(x, collections.abc.Iterable)` to gate
iter-protocol fast paths, `isinstance(x, collections.abc.Hashable)`
to admit a key into a hash-keyed cache, `isinstance(x,
collections.abc.Sized)` to decide whether `len()` is safe, and
`isinstance(x, collections.abc.Container)` to route `in`-style
membership probes. The canonical hot-path idiom is to read those
ABC names directly off the `collections.abc` module on every call
rather than caching a local — keeps the call site robust against
late-binding patterns (test monkey-patching, ABC-registry plugins,
backend-swap fixtures). That per-iter module-attribute quadruple
read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x —
CPython's `collections.abc.Iterable` / `collections.abc.Hashable`
/ `collections.abc.Sized` / `collections.abc.Container` are
top-level module-dict probes returning the ABC class objects on
3.12). Mamba's shim returns the same identity-stable sentinel
objects directly from a dense constant table in the
`collections.abc` module-attribute resolver, short-circuiting
CPython's module-dict probe chain for read-only ABC sentinels.

Workload: 10_000 paired reads of `collections.abc.Iterable`,
`collections.abc.Hashable`, `collections.abc.Sized`, and
`collections.abc.Container` per iteration, compared by identity
(`is`) against the hoisted baseline references taken once before
the loop. The accumulator increments when all four reads resolve
to the identical sentinel objects; a misread (different identity /
wrong binding) would immediately fail the correctness assert and
dead-code elimination of any read would leave `acc != ITERS`.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import collections.abc as _ca

# Hoist baseline references to the canonical ABC sentinels once
# before the loop. The hot path re-reads the module attribute on
# every iter so the bench actually exercises the module-attribute
# resolver — the `is` compare against the hoisted baseline is the
# correctness probe.
_ITERABLE_BASELINE = _ca.Iterable
_HASHABLE_BASELINE = _ca.Hashable
_SIZED_BASELINE = _ca.Sized
_CONTAINER_BASELINE = _ca.Container

ITERS = 10_000

acc = 0
for _ in range(ITERS):
    i = _ca.Iterable
    h = _ca.Hashable
    s = _ca.Sized
    c = _ca.Container
    # Accumulator readback prevents DCE — every iteration must
    # resolve to the identical sentinel objects bound at the
    # `collections.abc.Iterable` / `collections.abc.Hashable` /
    # `collections.abc.Sized` / `collections.abc.Container` module
    # slots.
    if (i is _ITERABLE_BASELINE
            and h is _HASHABLE_BASELINE
            and s is _SIZED_BASELINE
            and c is _CONTAINER_BASELINE):
        acc = acc + 1

# Correctness: every iteration must read back the canonical ABC
# sentinels via the module-attribute resolver. acc == ITERS or we
# have a regression in mamba's collections.abc module-attribute
# table.
assert acc - ITERS == 0, f"collections.abc module-attribute read acc drift: acc={acc} expected={ITERS}"
print("abc_type_read_hot:", acc)
