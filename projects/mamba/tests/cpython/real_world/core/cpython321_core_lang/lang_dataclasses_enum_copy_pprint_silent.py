# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_dataclasses_enum_copy_pprint_silent"
# subject = "cpython321.lang_dataclasses_enum_copy_pprint_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_dataclasses_enum_copy_pprint_silent.py"
# status = "filled"
# ///
"""cpython321.lang_dataclasses_enum_copy_pprint_silent: execute CPython 3.12 seed lang_dataclasses_enum_copy_pprint_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(dataclasses, 'is_dataclass')`
# (the documented "dataclasses exposes the is_dataclass predicate"
# — mamba returns False), `hasattr(dataclasses, 'MISSING')` (the
# documented "dataclasses exposes the MISSING sentinel" — mamba
# returns False), `hasattr(dataclasses, 'Field')` (the documented
# "dataclasses exposes the Field class" — mamba returns False),
# `hasattr(enum, 'EnumMeta')` (the documented "enum exposes the
# EnumMeta metaclass alias" — mamba returns False), `hasattr(enum,
# 'global_enum')` (the documented "enum exposes the global_enum
# decorator" — mamba returns False), `enum.Enum.__name__` (the
# documented "Enum class has __name__ == 'Enum'" — mamba returns
# None), `Color.RED.value` (the documented "Enum member exposes
# its declared value via .value" — mamba returns None), `len
# (Color)` (the documented "len(EnumClass) == number of declared
# members" — mamba returns 5 for a 3-member enum), `hasattr
# (pprint, 'PrettyPrinter')` (the documented "pprint exposes the
# PrettyPrinter class" — mamba returns False), and `pprint.pformat
# ([1, 2, 3])` (the documented "short collections are formatted on
# one line — returns '[1, 2, 3]'" — mamba returns multi-line '[\n
# 1,\n 2,\n 3\n]').
# Ten-pack pinned to atomic 269.
#
# Behavioral edges that CONFORM on mamba (dataclasses — hasattr
# dataclass/field/fields/asdict/astuple. enum — hasattr Enum/IntEnum/
# Flag/IntFlag/StrEnum/auto/unique/EnumType. copy — hasattr copy/
# deepcopy/Error + copy.copy of list/dict + copy.deepcopy of nested
# list value contracts. pprint — hasattr pprint/pformat) are
# covered in the matching pass fixture
# `test_dataclasses_enum_copy_pprint_value_ops`.
import dataclasses
import enum
import pprint


class _Color(enum.Enum):
    RED = 1
    GREEN = 2
    BLUE = 3


_ledger: list[int] = []

# 1) hasattr(dataclasses, 'is_dataclass') — predicate
#    (mamba: returns False)
assert hasattr(dataclasses, "is_dataclass") == True; _ledger.append(1)

# 2) hasattr(dataclasses, 'MISSING') — sentinel
#    (mamba: returns False)
assert hasattr(dataclasses, "MISSING") == True; _ledger.append(1)

# 3) hasattr(dataclasses, 'Field') — Field class
#    (mamba: returns False)
assert hasattr(dataclasses, "Field") == True; _ledger.append(1)

# 4) hasattr(enum, 'EnumMeta') — metaclass alias
#    (mamba: returns False)
assert hasattr(enum, "EnumMeta") == True; _ledger.append(1)

# 5) hasattr(enum, 'global_enum') — decorator
#    (mamba: returns False)
assert hasattr(enum, "global_enum") == True; _ledger.append(1)

# 6) enum.Enum.__name__ == 'Enum' — class identity
#    (mamba: returns None)
assert enum.Enum.__name__ == "Enum"; _ledger.append(1)

# 7) _Color.RED.value == 1 — member value accessor
#    (mamba: returns None)
assert _Color.RED.value == 1; _ledger.append(1)

# 8) len(_Color) == 3 — declared-member count
#    (mamba: returns 5 — counts extra slots)
assert len(_Color) == 3; _ledger.append(1)

# 9) hasattr(pprint, 'PrettyPrinter') — PrettyPrinter class
#    (mamba: returns False)
assert hasattr(pprint, "PrettyPrinter") == True; _ledger.append(1)

# 10) pprint.pformat([1, 2, 3]) — short list one-liner
#     (mamba: returns multi-line '[\n 1,\n 2,\n 3\n]')
assert pprint.pformat([1, 2, 3]) == "[1, 2, 3]"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_dataclasses_enum_copy_pprint_silent {sum(_ledger)} asserts")
