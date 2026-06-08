# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_dataclasses_enum_types_typing_silent"
# subject = "cpython321.lang_dataclasses_enum_types_typing_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_dataclasses_enum_types_typing_silent.py"
# status = "filled"
# ///
"""cpython321.lang_dataclasses_enum_types_typing_silent: execute CPython 3.12 seed lang_dataclasses_enum_types_typing_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the `dataclasses` /
# `enum` / `types` / `typing` four-pack pinned to atomic 212:
# `dataclasses` (the documented
# `hasattr(dataclasses, "make_dataclass") / "replace" /
# "is_dataclass" / "MISSING" / "FrozenInstanceError" /
# "InitVar" / "Field" / "KW_ONLY" == True` extended
# hasattr surface), `enum` (the documented
# `hasattr(enum, "EnumMeta") / "ReprEnum" / "EnumCheck" /
# "FlagBoundary" / "verify" == True` extended hasattr
# surface), `types` (the documented
# `type(types.SimpleNamespace(a=1, b=2)).__name__ ==
# "SimpleNamespace"` constructor-identity value contract +
# the documented `types.SimpleNamespace(a=1, b=2).a == 1`
# attribute-access value contract), and `typing` (the
# documented `hasattr(typing, "Annotated") / "NewType" /
# "overload" / "get_args" / "get_origin" / "TypeAlias" /
# "ParamSpec" / "Self" / "Never" / "LiteralString" /
# "Concatenate" / "Unpack" / "TypeVarTuple" / "Required"
# / "NotRequired" == True` extended hasattr surface + the
# documented `str(typing.Optional[int]) ==
# "typing.Optional[int]"` / `str(typing.Union[int, str])
# == "typing.Union[int, str]"` generic-alias-repr value
# contract).
#
# Behavioral edges that CONFORM on mamba
# (dataclasses `dataclass` / `field` / `fields` / `asdict`
# / `astuple` hasattr surface, enum `Enum` / `IntEnum` /
# `Flag` / `IntFlag` / `StrEnum` / `auto` / `unique` /
# `EnumType` hasattr surface, types full hasattr surface
# minus SimpleNamespace value contract, typing `Any` /
# `List` / `Dict` / `Tuple` / `Set` / `FrozenSet` /
# `Optional` / `Union` / `Callable` / `TypeVar` /
# `Generic` / `Protocol` / `TYPE_CHECKING` / `ClassVar` /
# `Final` / `Literal` / `TypedDict` / `NamedTuple` /
# `cast` / `get_type_hints` hasattr surface +
# `typing.List[int]` runtime subscriptability) are
# covered in the matching pass fixture
# `test_dataclasses_enum_types_typing_abc_value_ops`.
from typing import Any
import dataclasses as _dataclasses_mod
import enum as _enum_mod
import types as _types_mod
import typing as _typing_mod

dataclasses: Any = _dataclasses_mod
enum: Any = _enum_mod
types: Any = _types_mod
typing: Any = _typing_mod


_ledger: list[int] = []

# 1) dataclasses — extended module hasattr surface
#    (mamba: make_dataclass / replace / is_dataclass / MISSING
#    / FrozenInstanceError / InitVar / Field / KW_ONLY all
#    False)
assert hasattr(dataclasses, "make_dataclass") == True; _ledger.append(1)
assert hasattr(dataclasses, "replace") == True; _ledger.append(1)
assert hasattr(dataclasses, "is_dataclass") == True; _ledger.append(1)
assert hasattr(dataclasses, "MISSING") == True; _ledger.append(1)
assert hasattr(dataclasses, "FrozenInstanceError") == True; _ledger.append(1)
assert hasattr(dataclasses, "InitVar") == True; _ledger.append(1)
assert hasattr(dataclasses, "Field") == True; _ledger.append(1)
assert hasattr(dataclasses, "KW_ONLY") == True; _ledger.append(1)

# 2) enum — extended module hasattr surface
#    (mamba: EnumMeta / ReprEnum / EnumCheck / FlagBoundary /
#    verify all False)
assert hasattr(enum, "EnumMeta") == True; _ledger.append(1)
assert hasattr(enum, "ReprEnum") == True; _ledger.append(1)
assert hasattr(enum, "EnumCheck") == True; _ledger.append(1)
assert hasattr(enum, "FlagBoundary") == True; _ledger.append(1)
assert hasattr(enum, "verify") == True; _ledger.append(1)

# 3) types — SimpleNamespace constructor / attribute-access
#    value contract
#    (mamba: types.SimpleNamespace(a=1, b=2) returns a `dict`
#    instead of a SimpleNamespace + `.a` / `.b` attribute
#    access on the result raises AttributeError)
_sn = types.SimpleNamespace(a=1, b=2)
assert type(_sn).__name__ == "SimpleNamespace"; _ledger.append(1)
assert _sn.a == 1; _ledger.append(1)
assert _sn.b == 2; _ledger.append(1)

# 4) typing — extended module hasattr surface
#    (mamba: Annotated / NewType / overload / get_args /
#    get_origin / TypeAlias / ParamSpec / Self / Never /
#    LiteralString / Concatenate / Unpack / TypeVarTuple /
#    Required / NotRequired all False)
assert hasattr(typing, "Annotated") == True; _ledger.append(1)
assert hasattr(typing, "NewType") == True; _ledger.append(1)
assert hasattr(typing, "overload") == True; _ledger.append(1)
assert hasattr(typing, "get_args") == True; _ledger.append(1)
assert hasattr(typing, "get_origin") == True; _ledger.append(1)
assert hasattr(typing, "TypeAlias") == True; _ledger.append(1)
assert hasattr(typing, "ParamSpec") == True; _ledger.append(1)
assert hasattr(typing, "Self") == True; _ledger.append(1)
assert hasattr(typing, "Never") == True; _ledger.append(1)
assert hasattr(typing, "LiteralString") == True; _ledger.append(1)
assert hasattr(typing, "Concatenate") == True; _ledger.append(1)
assert hasattr(typing, "Unpack") == True; _ledger.append(1)
assert hasattr(typing, "TypeVarTuple") == True; _ledger.append(1)
assert hasattr(typing, "Required") == True; _ledger.append(1)
assert hasattr(typing, "NotRequired") == True; _ledger.append(1)

# 5) typing — generic-alias-repr value contract
#    (mamba: str(typing.Optional[int]) collapses to "None" +
#    str(typing.Union[int, str]) collapses to "None")
assert str(typing.Optional[int]) == "typing.Optional[int]"; _ledger.append(1)
assert str(typing.Union[int, str]) == "typing.Union[int, str]"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_dataclasses_enum_types_typing_silent {sum(_ledger)} asserts")
