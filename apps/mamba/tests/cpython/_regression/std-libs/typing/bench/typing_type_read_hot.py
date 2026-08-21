"""Hot-loop bench for lightweight `typing` module-attribute reads (#1104).

End-user scenario: annotation-heavy frameworks repeatedly resolve stable
`typing` sentinels while building schemas, validators, and model fields.
This pin measures the import-time/runtime-lightweight surface promised by
#1104 without requiring full static typing semantics at runtime.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 10_000 paired reads of common `typing` runtime sentinels per
iteration, compared by identity against hoisted baseline references. The
identity checks are the correctness probe and keep the reads live.
"""

import typing as _typing

_ANY_BASELINE = _typing.Any
_OPTIONAL_BASELINE = _typing.Optional
_UNION_BASELINE = _typing.Union
_TYPEVAR_BASELINE = _typing.TypeVar
_GENERIC_BASELINE = _typing.Generic

ITERS = 10_000

acc = 0
for _ in range(ITERS):
    any_v = _typing.Any
    optional_v = _typing.Optional
    union_v = _typing.Union
    typevar_v = _typing.TypeVar
    generic_v = _typing.Generic
    if (
        any_v is _ANY_BASELINE
        and optional_v is _OPTIONAL_BASELINE
        and union_v is _UNION_BASELINE
        and typevar_v is _TYPEVAR_BASELINE
        and generic_v is _GENERIC_BASELINE
    ):
        acc = acc + 1

assert acc - ITERS == 0, f"typing module-attribute read acc drift: acc={acc} expected={ITERS}"
print("typing_type_read_hot:", acc)
