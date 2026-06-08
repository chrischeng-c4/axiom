"""Hot-loop bench for `typing_extensions.Protocol` /
`typing_extensions.TypedDict` / `typing_extensions.runtime_checkable` /
`typing_extensions.override` module-attribute reads (#1494).

End-user scenario: any Python codebase that uses structural typing
(`Protocol`), declarative-shape dicts (`TypedDict`), runtime
isinstance-check protocols (`runtime_checkable`), or method-override
markers (`override`) re-imports these names from `typing_extensions`
at each call site rather than caching a local alias. Wrapper code
that decorates a class via `@typing_extensions.runtime_checkable` or
a method via `@typing_extensions.override` re-resolves these names
through the module's attribute table on every decoration. That
per-call module-attribute quad-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 these are a mix of own classes, re-exports from
`typing`, and decorator functions, all routed through the
`typing_extensions` module dict). Mamba's shim returns the same
identity-stable sentinels directly from a dense constant table in
the `typing_extensions` module-attribute resolver, short-circuiting
CPython's module-dict probe chain for read-only sentinels.

Workload: 20_000 paired reads of `Protocol`, `TypedDict`,
`runtime_checkable`, and `override` per iteration (ITERS scaled to
20_000 so 4 attrs x 20k = ~80k attr-reads per run, matching the
per-spawn budget of the 8-attr fixtures at 10_000 iters, the 2-attr
fixtures at 40_000 iters, and the 1-attr fixtures at 80_000 iters).
All four values are re-resolved from the `typing_extensions`
module-attribute table on every iter (not hoisted to a local) and
identity-compared against the hoisted baseline references; the
accumulator increments when all four reads resolve to identical
objects.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import typing_extensions as _te

_PROTOCOL_BASELINE = _te.Protocol
_TYPED_DICT_BASELINE = _te.TypedDict
_RUNTIME_CHECKABLE_BASELINE = _te.runtime_checkable
_OVERRIDE_BASELINE = _te.override

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = _te.Protocol
    b = _te.TypedDict
    c = _te.runtime_checkable
    d = _te.override
    if (a is _PROTOCOL_BASELINE
            and b is _TYPED_DICT_BASELINE
            and c is _RUNTIME_CHECKABLE_BASELINE
            and d is _OVERRIDE_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"typing_extensions module-attribute read acc drift: acc={acc} expected={ITERS}"
print("typing_extensions_type_read_hot:", acc)
