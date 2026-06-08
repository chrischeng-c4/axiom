"""Hot-loop bench for `dataclasses.dataclass` / `dataclasses.field` /
`dataclasses.fields` / `dataclasses.asdict` / `dataclasses.astuple`
module-attribute reads (#1446).

End-user scenario: dataclass-heavy code paths (config models, API
request/response schemas, serialization helpers) typically read the
`dataclasses` module-level callables on every site rather than
caching a local alias. Frameworks like attrs-style decorators, ORM
adapters, and validation libraries repeatedly resolve
`dataclasses.dataclass`, `dataclasses.field`, `dataclasses.fields`,
`dataclasses.asdict`, and `dataclasses.astuple` through the module's
attribute table. That per-call module-attribute quint-read is the
workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
CPython's `dataclasses.dataclass` family are top-level module-dict
probes on 3.12). Mamba's shim returns the same callable objects
directly from a dense constant table in the `dataclasses`
module-attribute resolver, short-circuiting CPython's module-dict
probe chain for read-only callable sentinels.

Workload: 10_000 paired reads of `dataclasses.dataclass`,
`dataclasses.field`, `dataclasses.fields`, `dataclasses.asdict`, and
`dataclasses.astuple` per iteration, compared by identity (`is`)
against the hoisted baseline references taken once before the loop.
The accumulator increments when all five reads resolve to the
identical callable objects; a misread (different identity / wrong
binding) would immediately fail the correctness assert and dead-code
elimination of any read would leave `acc != ITERS`.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import dataclasses as _dataclasses

# Hoist baseline references to the canonical callable objects once
# before the loop. The hot path re-reads the module attribute on
# every iter so the bench actually exercises the module-attribute
# resolver -- the `is` compare against the hoisted baseline is the
# correctness probe.
_DATACLASS_BASELINE = _dataclasses.dataclass
_FIELD_BASELINE = _dataclasses.field
_FIELDS_BASELINE = _dataclasses.fields
_ASDICT_BASELINE = _dataclasses.asdict
_ASTUPLE_BASELINE = _dataclasses.astuple

ITERS = 10_000

acc = 0
for _ in range(ITERS):
    dc = _dataclasses.dataclass
    fl = _dataclasses.field
    fs = _dataclasses.fields
    ad = _dataclasses.asdict
    at = _dataclasses.astuple
    # Accumulator readback prevents DCE -- every iteration must
    # resolve to the identical callable objects bound at the
    # `dataclasses.dataclass` / `dataclasses.field` /
    # `dataclasses.fields` / `dataclasses.asdict` /
    # `dataclasses.astuple` module slots.
    if (dc is _DATACLASS_BASELINE
            and fl is _FIELD_BASELINE
            and fs is _FIELDS_BASELINE
            and ad is _ASDICT_BASELINE
            and at is _ASTUPLE_BASELINE):
        acc = acc + 1

# Correctness: every iteration must read back the canonical callable
# objects via the module-attribute resolver. acc == ITERS or we have
# a regression in mamba's dataclasses module-attribute table.
assert acc - ITERS == 0, f"dataclasses module-attribute read acc drift: acc={acc} expected={ITERS}"
print("dataclasses_type_read_hot:", acc)
