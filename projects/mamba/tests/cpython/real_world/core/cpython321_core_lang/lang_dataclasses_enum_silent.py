# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_dataclasses_enum_silent"
# subject = "cpython321.lang_dataclasses_enum_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_dataclasses_enum_silent.py"
# status = "filled"
# ///
"""cpython321.lang_dataclasses_enum_silent: execute CPython 3.12 seed lang_dataclasses_enum_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(dataclasses, 'is_dataclass')`
# (the documented "dataclasses exposes is_dataclass predicate" —
# mamba returns False), `hasattr(dataclasses, 'MISSING')` (the
# documented "dataclasses exposes the MISSING sentinel" — mamba
# returns False), `hasattr(dataclasses, 'FrozenInstanceError')` (the
# documented "dataclasses exposes the FrozenInstanceError class" —
# mamba returns False), `hasattr(dataclasses, 'replace')` (the
# documented "dataclasses exposes the replace function" — mamba
# returns False), `Point(1, 2).x == 1` for a @dataclass-decorated
# class (the documented "@dataclass synthesises __init__ assigning
# positional args to fields" — mamba returns None — field attribute
# resolves to None placeholder), `type(Color).__name__ == 'EnumType'`
# (the documented "an Enum subclass is an instance of EnumType" —
# mamba returns 'str' — class object degrades to a string),
# `type(Color.RED).__name__ == 'Color'` (the documented "an Enum
# member is an instance of the Enum class" — mamba returns 'int' —
# member degrades to its bare value), `Color.RED.name == 'RED'` (the
# documented "Enum member .name returns the declared label" — mamba
# returns None — attribute resolves to None placeholder),
# `Color.RED.value == 1` (the documented "Enum member .value returns
# the declared value" — mamba returns None — attribute resolves to
# None placeholder), and `len(Color) == 2` (the documented "len(Enum)
# returns the count of declared members" — mamba returns 1 —
# only one member visible).
# Ten-pack pinned to atomic 301.
#
# Behavioral edges that CONFORM on mamba (dataclasses — hasattr
# dataclass/field/fields/asdict/astuple + decorated class instance
# type echoes class name. enum — hasattr Enum/IntEnum/Flag/IntFlag/
# StrEnum/auto/unique + member equality reflexive + member equality
# between distinct members False + IntEnum integer coercion
# arithmetic + int(member) of IntEnum) are covered in the matching
# pass fixture `test_dataclasses_enum_surface_value_ops`.
import dataclasses
import enum


_ledger: list[int] = []

# 1) hasattr(dataclasses, 'is_dataclass') — is_dataclass predicate
#    (mamba: returns False)
assert hasattr(dataclasses, "is_dataclass") == True; _ledger.append(1)

# 2) hasattr(dataclasses, 'MISSING') — MISSING sentinel
#    (mamba: returns False)
assert hasattr(dataclasses, "MISSING") == True; _ledger.append(1)

# 3) hasattr(dataclasses, 'FrozenInstanceError') — FrozenInstanceError class
#    (mamba: returns False)
assert hasattr(dataclasses, "FrozenInstanceError") == True; _ledger.append(1)

# 4) hasattr(dataclasses, 'replace') — replace function
#    (mamba: returns False)
assert hasattr(dataclasses, "replace") == True; _ledger.append(1)


@dataclasses.dataclass
class Point:
    x: int
    y: int


# 5) Point(1, 2).x == 1 — @dataclass __init__ assigns positional args
#    (mamba: returns None — field attribute resolves to None placeholder)
assert Point(1, 2).x == 1; _ledger.append(1)


class Color(enum.Enum):
    RED = 1
    BLUE = 2


# 6) type(Color).__name__ == 'EnumType' — Enum subclass is EnumType instance
#    (mamba: returns 'str' — class object degrades to a string)
assert type(Color).__name__ == "EnumType"; _ledger.append(1)

# 7) type(Color.RED).__name__ == 'Color' — member is Enum-class instance
#    (mamba: returns 'int' — member degrades to its bare value)
assert type(Color.RED).__name__ == "Color"; _ledger.append(1)

# 8) Color.RED.name == 'RED' — member .name returns declared label
#    (mamba: returns None — attribute resolves to None placeholder)
assert Color.RED.name == "RED"; _ledger.append(1)

# 9) Color.RED.value == 1 — member .value returns declared value
#    (mamba: returns None — attribute resolves to None placeholder)
assert Color.RED.value == 1; _ledger.append(1)

# 10) len(Color) == 2 — len(Enum) counts declared members
#     (mamba: returns 1 — only one member visible)
assert len(Color) == 2; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_dataclasses_enum_silent {sum(_ledger)} asserts")
