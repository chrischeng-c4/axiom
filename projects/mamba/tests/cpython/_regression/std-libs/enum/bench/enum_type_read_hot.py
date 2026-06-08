"""Hot-loop bench for `enum.Enum` / `enum.IntEnum` / `enum.StrEnum` /
`enum.Flag` / `enum.IntFlag` / `enum.auto` / `enum.unique` /
`enum.EnumMeta` module-attribute reads (#1448).

End-user scenario: hot code-generation / schema-binding code that
resolves enum-machinery constructors and decorators off the `enum`
module on every call site - e.g. a config-loader loop that calls
`enum.Enum` to mint a nominal-tag class, `enum.IntEnum` for
integer-backed wire codes, `enum.StrEnum` for stringly-typed
identifiers, `enum.Flag` for bitfield combinators, `enum.IntFlag`
for C-style bitmasks, `enum.auto` to assign monotonic member values,
`enum.unique` to guard against duplicate-value misconfigs, and
`enum.EnumMeta` to reflect on member tables. The canonical hot-path
idiom is to read those names directly off the `enum` module on every
call rather than caching a local - keeps the call site robust against
late-binding patterns (test monkey-patching, schema-plugin overrides,
runtime backend-swap fixtures). That per-iter module-attribute
octuple read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x -
CPython's `enum.Enum` / `enum.IntEnum` / `enum.StrEnum` /
`enum.Flag` / `enum.IntFlag` / `enum.auto` / `enum.unique` /
`enum.EnumMeta` are top-level module-dict probes returning the
canonical class / function objects on 3.12). Mamba's shim returns the
same identity-stable callables directly from a dense constant table
in the `enum` module-attribute resolver, short-circuiting CPython's
module-dict probe chain for read-only enum-machinery sentinels.

Workload: 10_000 paired reads of `enum.Enum`, `enum.IntEnum`,
`enum.StrEnum`, `enum.Flag`, `enum.IntFlag`, `enum.auto`,
`enum.unique`, and `enum.EnumMeta` per iteration, compared by identity
(`is`) against the hoisted baseline references taken once before the
loop. The accumulator increments when all eight reads resolve to the
identical callable objects; a misread (different identity / wrong
binding) would immediately fail the correctness assert and dead-code
elimination of any read would leave `acc != ITERS`.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import enum as _en

# Hoist baseline references to the canonical enum-machinery callables
# once before the loop. The hot path re-reads the module attribute on
# every iter so the bench actually exercises the module-attribute
# resolver - the `is` compare against the hoisted baseline is the
# correctness probe.
_ENUM_BASELINE = _en.Enum
_INTENUM_BASELINE = _en.IntEnum
_STRENUM_BASELINE = _en.StrEnum
_FLAG_BASELINE = _en.Flag
_INTFLAG_BASELINE = _en.IntFlag
_AUTO_BASELINE = _en.auto
_UNIQUE_BASELINE = _en.unique
_ENUMMETA_BASELINE = _en.EnumMeta

ITERS = 10_000

acc = 0
for _ in range(ITERS):
    e = _en.Enum
    i = _en.IntEnum
    s = _en.StrEnum
    f = _en.Flag
    ifl = _en.IntFlag
    a = _en.auto
    u = _en.unique
    m = _en.EnumMeta
    # Accumulator readback prevents DCE - every iteration must
    # resolve to the identical callable objects bound at the
    # `enum.Enum` / `enum.IntEnum` / `enum.StrEnum` / `enum.Flag` /
    # `enum.IntFlag` / `enum.auto` / `enum.unique` / `enum.EnumMeta`
    # module slots.
    if (e is _ENUM_BASELINE
            and i is _INTENUM_BASELINE
            and s is _STRENUM_BASELINE
            and f is _FLAG_BASELINE
            and ifl is _INTFLAG_BASELINE
            and a is _AUTO_BASELINE
            and u is _UNIQUE_BASELINE
            and m is _ENUMMETA_BASELINE):
        acc = acc + 1

# Correctness: every iteration must read back the canonical
# enum-machinery callables via the module-attribute resolver.
# acc == ITERS or we have a regression in mamba's enum
# module-attribute table.
assert acc - ITERS == 0, f"enum module-attribute read acc drift: acc={acc} expected={ITERS}"
print("enum_type_read_hot:", acc)
