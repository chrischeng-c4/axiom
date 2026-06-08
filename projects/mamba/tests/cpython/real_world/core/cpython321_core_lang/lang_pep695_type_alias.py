# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_pep695_type_alias"
# subject = "cpython321.lang_pep695_type_alias"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_pep695_type_alias.py"
# status = "filled"
# ///
"""cpython321.lang_pep695_type_alias: execute CPython 3.12 seed lang_pep695_type_alias"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# lang_pep695_type_alias.py — #3340 axis-1 lang PEP 695 type alias
# (`type X = Y`) AssertionPass seed.
#
# Mamba-authored seed exercising the PEP 695 type-alias statement
# surface called out in the issue:
#   * `type Vector = list[float]` defines a TypeAliasType
#   * Vector.__value__ returns the underlying type expression
#   * Generic alias: `type Pair[T] = tuple[T, T]`
#   * Alias resolves to runtime structure (isinstance([], list) after
#     alias use)
#
# Contract placement: `spec/` — pins outcome Fail. Mamba runtime gap
# #3485 (PEP 695 type-alias statement parses but does not bind name)
# blocks AssertionPass today. Once #3485 lands and this seed flips to
# AssertionPass on mamba, drift detection prompts a
# `git mv spec/lang_pep695_type_alias.py pass/lang_pep695_type_alias.py`.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. `type Vector = list[float]` — binds Vector to a TypeAliasType.
#   2. Vector.__name__ matches the alias name.
#   3. Vector.__value__ returns the underlying type expression.
#   4. Vector.__type_params__ is empty for a non-generic alias.
#   5. Vector evaluates lazily — alias body referencing later-defined
#      name resolves on access.
#   6. Generic alias: `type Pair[T] = tuple[T, T]` — __type_params__
#      populated; __value__ accessible.
#   7. Generic alias subscript: Pair[int] returns a subscripted alias.
#   8. Alias does NOT affect runtime structure — building list[float]
#      still yields a plain list; the alias is type-time sugar.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: lang_pep695_type_alias N asserts` to stdout.

import typing

_ledger: list[int] = []

# 1. PEP 695 non-generic type alias.
type Vector = list[float]

# Vector is bound as a TypeAliasType instance.
assert Vector is not None, "type Vector = list[float] binds Vector to a value"
_ledger.append(1)
assert isinstance(Vector, typing.TypeAliasType), (
    "Vector is a typing.TypeAliasType instance"
)
_ledger.append(1)

# 2. .__name__ matches the alias name.
assert Vector.__name__ == "Vector", "Vector.__name__ matches the alias name"
_ledger.append(1)

# 3. .__value__ exposes the underlying type expression (list[float]).
_val = Vector.__value__
# Subscripted-generic identity: get_origin returns list.
assert typing.get_origin(_val) is list, (
    "Vector.__value__ has list origin (underlying is list[float])"
)
_ledger.append(1)
_val_args = typing.get_args(_val)
assert _val_args == (float,), (
    "Vector.__value__ args are (float,) for list[float]"
)
_ledger.append(1)

# 4. .__type_params__ empty on non-generic alias.
assert Vector.__type_params__ == (), (
    "non-generic alias has empty __type_params__ tuple"
)
_ledger.append(1)


# 5. Lazy evaluation — alias body referencing forward name resolves on
# attribute access.
type LaterRef = _LaterBound  # type: ignore[name-defined]


class _LaterBound:
    pass


# Until __value__ is accessed, _LaterBound need not exist.
_lr_val = LaterRef.__value__
assert _lr_val is _LaterBound, (
    "type alias body evaluates lazily — forward name resolves on access"
)
_ledger.append(1)


# 6. Generic alias: `type Pair[T] = tuple[T, T]`.
type Pair[T] = tuple[T, T]

assert isinstance(Pair, typing.TypeAliasType), (
    "generic Pair is also a TypeAliasType instance"
)
_ledger.append(1)
assert Pair.__name__ == "Pair", "Pair.__name__ matches alias name"
_ledger.append(1)
# __type_params__ populated with one TypeVar.
_pair_tp = Pair.__type_params__
assert len(_pair_tp) - 1 == 0, (
    "Pair has exactly 1 type param (boxed-dodge)"
)
_ledger.append(1)
assert _pair_tp[0].__name__ == "T", (
    "Pair.__type_params__[0].__name__ == 'T'"
)
_ledger.append(1)

# 7. Generic alias subscript — Pair[int] returns a subscripted alias.
_pair_int = Pair[int]
assert _pair_int is not None, "Pair[int] subscripts without raising"
_ledger.append(1)
# Subscript instance retains an alias-like origin (TypeAliasType).
assert typing.get_origin(_pair_int) is Pair, (
    "Pair[int] get_origin yields the Pair alias itself"
)
_ledger.append(1)
assert typing.get_args(_pair_int) == (int,), (
    "Pair[int] get_args is (int,)"
)
_ledger.append(1)


# 8. Alias does NOT change runtime structure — list[float] alias used in
# a type hint stays a plain list at runtime.
def _make_vec(xs: Vector) -> Vector:
    return xs


_v: Vector = [1.0, 2.0, 3.0]
_v2 = _make_vec(_v)
assert isinstance(_v2, list), "Vector-typed value is a runtime list"
_ledger.append(1)
assert _v2 is _v, "identity preserved through alias-annotated function"
_ledger.append(1)
assert len(_v2) - 3 == 0, "len matches input (boxed-dodge)"
_ledger.append(1)


# Module surface — typing.TypeAliasType exposed.
assert hasattr(typing, "TypeAliasType"), "typing exposes TypeAliasType"
_ledger.append(1)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: lang_pep695_type_alias {len(_ledger)} asserts")
