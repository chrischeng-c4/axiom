"""Hot-loop bench for `weakref.ReferenceType` / `weakref.ProxyType`
module-attribute reads (#1466).

End-user scenario: introspection / dispatch paths that re-resolve
the weakref type families from the `weakref` module attribute table
on every event — `isinstance(x, weakref.ReferenceType)` is the
canonical predicate that downstream code (debuggers, repl pretty-
printers, type-keyed dispatch tables, structured loggers, GC
inspectors) routes through. The canonical hot-path idiom is to
read `weakref.ReferenceType` / `weakref.ProxyType` directly each
call rather than caching a local — keeps the call site robust
against module-reload patterns. That per-iter module-attribute
double-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x —
CPython's `weakref.ReferenceType` / `weakref.ProxyType` are top-
level module-dict probes returning the type objects on 3.12).
Mamba's shim returns the same type objects directly from a dense
constant table in the `weakref` module-attribute resolver, short-
circuiting CPython's module-dict probe chain for read-only type
sentinels.

Workload: 10_000 paired reads of `weakref.ReferenceType` and
`weakref.ProxyType` per iteration, compared by identity (`is`)
against the hoisted baseline references taken once before the
loop. The accumulator increments when both reads resolve to the
identical type objects; a misread (different identity / wrong
binding) immediately fails the correctness assert and dead-code
elimination of either read would leave `acc != ITERS`.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import weakref as _weakref

# Hoist baseline references to the canonical type objects once
# before the loop. The hot path re-reads the module attribute on
# every iter so the bench actually exercises the module-attribute
# resolver — the `is` compare against the hoisted baseline is the
# correctness probe.
_REF_BASELINE = _weakref.ReferenceType
_PROXY_BASELINE = _weakref.ProxyType

ITERS = 10_000

acc = 0
for _ in range(ITERS):
    r = _weakref.ReferenceType
    p = _weakref.ProxyType
    # Accumulator readback prevents DCE — every iteration must
    # resolve to the identical type objects bound at the
    # `weakref.ReferenceType` / `weakref.ProxyType` module slots.
    if r is _REF_BASELINE and p is _PROXY_BASELINE:
        acc = acc + 1

# Correctness: every iteration must read back the canonical type
# objects via the module-attribute resolver. acc == ITERS or we have
# a regression in mamba's weakref module-attribute table.
assert acc - ITERS == 0, f"weakref module-attribute read acc drift: acc={acc} expected={ITERS}"
print("proxy_type_name_read_hot:", acc)
