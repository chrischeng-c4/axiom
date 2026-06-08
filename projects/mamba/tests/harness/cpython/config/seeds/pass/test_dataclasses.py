# test_dataclasses.py — #3442 axis-1 stdlib dataclasses AssertionPass seed.
#
# Mamba-authored seed exercising the `dataclasses` module surface called
# out in the issue:
#   @dataclass eq/order/frozen, field default_factory, asdict, astuple,
#   replace.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + public surface (hasattr).
#   2. @dataclass minimal — __init__ binds positional/keyword args; eq
#      yields equality by field.
#   3. @dataclass(order=True) — generated __lt__/__le__ over the tuple
#      of fields.
#   4. @dataclass(frozen=True) — assignment to an instance field raises
#      FrozenInstanceError.
#   5. field(default_factory=list) — distinct list per instance.
#   6. asdict / astuple — recursive conversion through nested dataclasses.
#   7. replace — returns a copy with overridden fields.
#   8. fields() — returns a tuple of Field descriptors keyed by name.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: dataclasses N asserts` to stdout.

import dataclasses

_ledger: list[int] = []


@dataclasses.dataclass
class _Point:
    x: int
    y: int


@dataclasses.dataclass(order=True)
class _OrderedPoint:
    x: int
    y: int


@dataclasses.dataclass(frozen=True)
class _FrozenPoint:
    x: int
    y: int


@dataclasses.dataclass
class _Bag:
    items: list = dataclasses.field(default_factory=list)
    tags: dict = dataclasses.field(default_factory=dict)


@dataclasses.dataclass
class _Inner:
    label: str
    value: int


@dataclasses.dataclass
class _Outer:
    name: str
    inner: _Inner
    extras: list


# 1. Module identity + public surface.
assert dataclasses.__name__ == "dataclasses", "dataclasses.__name__"
_ledger.append(1)
assert hasattr(dataclasses, "dataclass"), "exposes dataclass decorator"
_ledger.append(1)
assert hasattr(dataclasses, "field"), "exposes field"
_ledger.append(1)
assert hasattr(dataclasses, "asdict"), "exposes asdict"
_ledger.append(1)
assert hasattr(dataclasses, "astuple"), "exposes astuple"
_ledger.append(1)
assert hasattr(dataclasses, "replace"), "exposes replace"
_ledger.append(1)
assert hasattr(dataclasses, "fields"), "exposes fields"
_ledger.append(1)
assert hasattr(dataclasses, "is_dataclass"), "exposes is_dataclass"
_ledger.append(1)
assert hasattr(dataclasses, "FrozenInstanceError"), "exposes FrozenInstanceError"
_ledger.append(1)

# 2. @dataclass minimal — __init__ + eq.
_p1 = _Point(1, 2)
assert _p1.x == 1, "@dataclass __init__ binds x"
_ledger.append(1)
assert _p1.y == 2, "@dataclass __init__ binds y"
_ledger.append(1)
_p2 = _Point(1, 2)
assert _p1 == _p2, "@dataclass eq is by-field equality"
_ledger.append(1)
_p3 = _Point(1, 3)
assert _p1 != _p3, "@dataclass eq distinguishes unequal field values"
_ledger.append(1)
# is_dataclass recognises the class + instance.
assert dataclasses.is_dataclass(_Point) == True, "is_dataclass(class) is True"
_ledger.append(1)
assert dataclasses.is_dataclass(_p1) == True, "is_dataclass(instance) is True"
_ledger.append(1)
assert dataclasses.is_dataclass(42) == False, "is_dataclass(int) is False"
_ledger.append(1)

# 3. order=True — __lt__/__le__ over field tuple.
_o1 = _OrderedPoint(1, 2)
_o2 = _OrderedPoint(1, 3)
_o3 = _OrderedPoint(2, 0)
assert _o1 < _o2, "(1,2) < (1,3) by field-tuple lexicographic order"
_ledger.append(1)
assert _o2 < _o3, "(1,3) < (2,0) — x compared first"
_ledger.append(1)
assert _o1 <= _o1, "ordered <= reflexive"
_ledger.append(1)
assert _o3 > _o2, "(2,0) > (1,3) via __gt__"
_ledger.append(1)

# 4. frozen=True — assignment raises FrozenInstanceError.
_fp = _FrozenPoint(7, 8)
assert _fp.x == 7, "frozen instance binds x"
_ledger.append(1)
_raised = False
try:
    _fp.x = 99  # type: ignore[misc]
except dataclasses.FrozenInstanceError:
    _raised = True
assert _raised == True, "frozen assignment raises FrozenInstanceError"
_ledger.append(1)
# Re-read to confirm the value did not mutate.
assert _fp.x == 7, "frozen field value unchanged after raised assignment"
_ledger.append(1)

# 5. default_factory — distinct mutable defaults per instance.
_b1 = _Bag()
_b2 = _Bag()
assert _b1.items == [], "default_factory=list yields empty list"
_ledger.append(1)
assert _b2.items == [], "second default_factory=list yields empty list"
_ledger.append(1)
assert _b1.items is not _b2.items, "default_factory yields a distinct object per instance"
_ledger.append(1)
_b1.items.append(42)
assert _b1.items == [42], "mutation visible on b1"
_ledger.append(1)
assert _b2.items == [], "default_factory isolated b2 from b1 mutation"
_ledger.append(1)

# 6. asdict / astuple — recursive over nested dataclasses.
_o = _Outer(name="root", inner=_Inner(label="leaf", value=99), extras=[1, 2, 3])
_d = dataclasses.asdict(_o)
assert isinstance(_d, dict), "asdict returns a dict"
_ledger.append(1)
assert _d["name"] == "root", "asdict carries top-level field"
_ledger.append(1)
assert isinstance(_d["inner"], dict), "asdict recurses into nested dataclass"
_ledger.append(1)
assert _d["inner"]["label"] == "leaf", "asdict recurses to inner.label"
_ledger.append(1)
assert _d["inner"]["value"] == 99, "asdict recurses to inner.value"
_ledger.append(1)
assert _d["extras"] == [1, 2, 3], "asdict preserves list-of-scalars"
_ledger.append(1)
_t = dataclasses.astuple(_o)
assert isinstance(_t, tuple), "astuple returns a tuple"
_ledger.append(1)
assert _t[0] == "root", "astuple[0] is the first field"
_ledger.append(1)
# Inner dataclass is itself converted to a tuple recursively.
assert _t[1] == ("leaf", 99), "astuple recurses into nested dataclass tuple"
_ledger.append(1)
assert _t[2] == [1, 2, 3], "astuple preserves list field"
_ledger.append(1)

# 7. replace — overridden field with a copy.
_p4 = dataclasses.replace(_p1, y=999)
assert _p4.x == 1, "replace preserves un-overridden field"
_ledger.append(1)
assert _p4.y == 999, "replace overrides 'y'"
_ledger.append(1)
assert _p1.y == 2, "replace does not mutate the original"
_ledger.append(1)
# replace on frozen — supported.
_fp2 = dataclasses.replace(_fp, x=42)
assert _fp2.x == 42, "replace on frozen returns a new frozen instance"
_ledger.append(1)
assert _fp.x == 7, "frozen original still has original x"
_ledger.append(1)

# 8. fields() — tuple of Field descriptors.
_fs = dataclasses.fields(_Point)
assert isinstance(_fs, tuple), "fields() returns a tuple"
_ledger.append(1)
_field_names = [f.name for f in _fs]
assert _field_names == ["x", "y"], "fields() preserves declaration order"
_ledger.append(1)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: dataclasses {len(_ledger)} asserts")
