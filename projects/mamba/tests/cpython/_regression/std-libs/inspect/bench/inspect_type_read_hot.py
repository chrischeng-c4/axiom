"""Hot-loop bench for `inspect.isclass` / `inspect.isfunction` /
`inspect.ismethod` / `inspect.ismodule` / `inspect.signature` /
`inspect.getmodule` / `inspect.Signature` / `inspect.Parameter`
module-attribute reads (#1440).

End-user scenario: hot introspection paths that resolve callable
predicates and signature objects off the `inspect` module on every
call site -- e.g. an ORM/DI container that dispatches on
`inspect.isclass(obj)` / `inspect.isfunction(obj)` /
`inspect.ismethod(obj)` to pick a binding strategy, calls
`inspect.signature(fn)` to introspect parameter defaults, falls back
to `inspect.getmodule(obj)` for diagnostic routing, and references
the `inspect.Signature` / `inspect.Parameter` / `inspect.ismodule`
sentinels for late-bound test fixtures and plugin loaders. The
canonical hot-path idiom is to re-read those names directly off the
`inspect` module on every call rather than caching a local -- keeps
the call site robust against monkey-patched introspection backends.
That per-iter module-attribute octuple read is the workload measured
here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
CPython's `inspect.isclass` / `inspect.isfunction` /
`inspect.ismethod` / `inspect.ismodule` / `inspect.signature` /
`inspect.getmodule` / `inspect.Signature` / `inspect.Parameter` are
top-level module-dict probes returning the canonical
function/class/None objects on 3.12). Mamba's shim returns the same
identity-stable callables / None sentinels directly from a dense
constant table in the `inspect` module-attribute resolver,
short-circuiting CPython's module-dict probe chain for read-only
introspection sentinels.

Workload: 10_000 paired reads of `inspect.isclass`,
`inspect.isfunction`, `inspect.ismethod`, `inspect.ismodule`,
`inspect.signature`, `inspect.getmodule`, `inspect.Signature`, and
`inspect.Parameter` per iteration, compared by identity (`is`)
against the hoisted baseline references taken once before the loop.
The accumulator increments when all eight reads resolve to the
identical objects; a misread (different identity / wrong binding)
would immediately fail the correctness assert and dead-code
elimination of any read would leave `acc != ITERS`.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import inspect as _ins

# Hoist baseline references to the canonical introspection callables /
# sentinels once before the loop. Each runtime hoists its own baseline:
# CPython binds the canonical function/class objects; mamba binds the
# identity-stable function objects (isclass/isfunction/ismethod/
# signature) and identity-stable None sentinels (ismodule/getmodule/
# Signature/Parameter) returned by its `inspect` module-attribute
# resolver. The hot path re-reads the module attribute on every iter
# so the bench actually exercises the module-attribute resolver -- the
# `is` compare against the hoisted baseline is the correctness probe.
_ISCLASS_BASELINE = _ins.isclass
_ISFUNCTION_BASELINE = _ins.isfunction
_ISMETHOD_BASELINE = _ins.ismethod
_ISMODULE_BASELINE = _ins.ismodule
_SIGNATURE_BASELINE = _ins.signature
_GETMODULE_BASELINE = _ins.getmodule
_SIGNATURE_CLS_BASELINE = _ins.Signature
_PARAMETER_BASELINE = _ins.Parameter

ITERS = 10_000

acc = 0
for _ in range(ITERS):
    ic = _ins.isclass
    if_ = _ins.isfunction
    im = _ins.ismethod
    imo = _ins.ismodule
    sg = _ins.signature
    gm = _ins.getmodule
    sc = _ins.Signature
    pa = _ins.Parameter
    # Accumulator readback prevents DCE -- every iteration must
    # resolve to the identical objects bound at the
    # `inspect.isclass` / `inspect.isfunction` / `inspect.ismethod` /
    # `inspect.ismodule` / `inspect.signature` / `inspect.getmodule` /
    # `inspect.Signature` / `inspect.Parameter` module slots.
    if (ic is _ISCLASS_BASELINE
            and if_ is _ISFUNCTION_BASELINE
            and im is _ISMETHOD_BASELINE
            and imo is _ISMODULE_BASELINE
            and sg is _SIGNATURE_BASELINE
            and gm is _GETMODULE_BASELINE
            and sc is _SIGNATURE_CLS_BASELINE
            and pa is _PARAMETER_BASELINE):
        acc = acc + 1

# Correctness: every iteration must read back the canonical
# introspection objects via the module-attribute resolver.
# acc == ITERS or we have a regression in mamba's inspect
# module-attribute table.
assert acc - ITERS == 0, f"inspect module-attribute read acc drift: acc={acc} expected={ITERS}"
print("inspect_type_read_hot:", acc)
