# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_decimal_fraction_enum_dataclass_typing_silent"
# subject = "cpython321.lang_decimal_fraction_enum_dataclass_typing_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_decimal_fraction_enum_dataclass_typing_silent.py"
# status = "filled"
# ///
"""cpython321.lang_decimal_fraction_enum_dataclass_typing_silent: execute CPython 3.12 seed lang_decimal_fraction_enum_dataclass_typing_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the `decimal.Decimal` /
# `fractions.Fraction` / `enum.Enum` / `dataclasses` /
# `typing` collection-ABC seven-pack pinned to atomic 236:
# `decimal.Decimal("1.1")` / `decimal.Decimal(42)` /
# `Decimal("1.1") + Decimal("2.2")` / `Decimal("1.0") ==
# Decimal("1.00")` (the documented "Decimal stringifies to its
# decimal repr, arithmetic returns the mathematically-correct
# Decimal, and trailing-zero variants compare equal" value
# contract — mamba's `Decimal(...)` constructor returns the
# boxed-handle integer so `str(...)` is the handle integer as
# a string (e.g. '70368744177664') and arithmetic on them
# returns garbage), `decimal.Context / getcontext / setcontext
# / ROUND_HALF_UP / ROUND_HALF_EVEN / InvalidOperation /
# DivisionByZero` (the documented module-level surface — mamba's
# `decimal` module dict does not expose any of them so
# `hasattr(...)` collapses to False), `fractions.Fraction(1, 2)
# / (3, 4) / from-float / from-str` (the documented "Fraction
# stringifies to 'num/den'" value contract — mamba's
# `Fraction(...)` constructor returns the boxed-handle integer
# so `str(...)` is '1099511627776' etc.), `enum.Enum` value /
# name / iteration (the documented "Enum member exposes .value
# and .name and iterating the class yields its members in
# definition order" value contract — mamba's `Color.RED.value`
# and `Color.RED.name` silently return None and iterating
# yields extra None entries instead of the three defined
# members), `dataclasses.is_dataclass / replace /
# FrozenInstanceError / MISSING` (the documented top-level
# surface — mamba's `dataclasses` module dict does not expose
# any of them), and `typing.Iterable / Sequence / Mapping`
# (the documented collection-ABC re-exports — mamba's `typing`
# module dict does not expose them even though Iterator and
# the other typing ABCs are present).
#
# Behavioral edges that CONFORM on mamba (json dumps/loads
# value ops + JSONDecodeError/JSONEncoder/JSONDecoder + dump/
# load surface, struct pack/unpack/calcsize + Struct/pack_into/
# unpack_from/iter_unpack/error, typing Optional/List/Dict/
# Tuple/Set/FrozenSet/Callable/Union/Any/ClassVar/Final/Type/
# TypeVar/Generic/Protocol/cast/get_type_hints/NamedTuple/
# TypedDict/Iterator, weakref ref/proxy/WeakValueDictionary/
# WeakKeyDictionary/WeakSet/finalize, abc ABC/ABCMeta/
# abstractmethod/abstractclassmethod/abstractstaticmethod/
# abstractproperty, enum Enum/IntEnum/Flag/IntFlag/auto/unique
# class binding, dataclasses dataclass/field/asdict/astuple/
# fields class binding, fractions Fraction.numerator/
# denominator + class binding, decimal Decimal class binding)
# are covered in the matching pass fixture
# `test_json_struct_typing_weakref_abc_value_ops`.
from typing import Any
import decimal as _decimal_mod
import fractions as _fractions_mod
import enum as _enum_mod
import dataclasses as _dataclasses_mod
import typing as _typing_mod

decimal_mod: Any = _decimal_mod
fractions_mod: Any = _fractions_mod
enum_mod: Any = _enum_mod
dataclasses_mod: Any = _dataclasses_mod
typing_mod: Any = _typing_mod


_ledger: list[int] = []

# 1) Decimal value contracts — decimal stringifies, arithmetic, compare
#    (mamba: constructor returns boxed-handle int)
assert str(decimal_mod.Decimal("1.1")) == "1.1"; _ledger.append(1)
assert str(decimal_mod.Decimal(42)) == "42"; _ledger.append(1)
assert str(decimal_mod.Decimal("1.1") + decimal_mod.Decimal("2.2")) == "3.3"; _ledger.append(1)
assert (decimal_mod.Decimal("1.0") == decimal_mod.Decimal("1.00")) == True; _ledger.append(1)

# 2) decimal module-level surface
#    (mamba: missing)
assert hasattr(decimal_mod, "Context") == True; _ledger.append(1)
assert hasattr(decimal_mod, "getcontext") == True; _ledger.append(1)
assert hasattr(decimal_mod, "setcontext") == True; _ledger.append(1)
assert hasattr(decimal_mod, "ROUND_HALF_UP") == True; _ledger.append(1)
assert hasattr(decimal_mod, "ROUND_HALF_EVEN") == True; _ledger.append(1)
assert hasattr(decimal_mod, "InvalidOperation") == True; _ledger.append(1)
assert hasattr(decimal_mod, "DivisionByZero") == True; _ledger.append(1)

# 3) Fraction value contracts — Fraction stringifies to 'num/den'
#    (mamba: constructor returns boxed-handle int)
assert str(fractions_mod.Fraction(1, 2)) == "1/2"; _ledger.append(1)
assert str(fractions_mod.Fraction(3, 4)) == "3/4"; _ledger.append(1)
assert str(fractions_mod.Fraction(0.25)) == "1/4"; _ledger.append(1)
assert str(fractions_mod.Fraction("3/4")) == "3/4"; _ledger.append(1)


# 4) Enum value contracts — .value, .name, and iteration order
#    (mamba: .value and .name silently return None; iteration yields
#    extra None entries)
class _Color(enum_mod.Enum):
    RED = 1
    GREEN = 2
    BLUE = 3


_Color_any: Any = _Color
_red_val: Any = _Color_any.RED.value
assert _red_val == 1; _ledger.append(1)
_red_name: Any = _Color_any.RED.name
assert _red_name == "RED"; _ledger.append(1)
assert [_c.name for _c in _Color_any] == ["RED", "GREEN", "BLUE"]; _ledger.append(1)

# 5) dataclasses module-level surface
#    (mamba: missing)
assert hasattr(dataclasses_mod, "is_dataclass") == True; _ledger.append(1)
assert hasattr(dataclasses_mod, "replace") == True; _ledger.append(1)
assert hasattr(dataclasses_mod, "FrozenInstanceError") == True; _ledger.append(1)
assert hasattr(dataclasses_mod, "MISSING") == True; _ledger.append(1)

# 6) typing collection-ABC re-exports
#    (mamba: missing even though Iterator and other typing classes are
#    present)
assert hasattr(typing_mod, "Iterable") == True; _ledger.append(1)
assert hasattr(typing_mod, "Sequence") == True; _ledger.append(1)
assert hasattr(typing_mod, "Mapping") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_decimal_fraction_enum_dataclass_typing_silent {sum(_ledger)} asserts")
