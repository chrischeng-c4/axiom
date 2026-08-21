# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `isinstance(1, numbers.Number)`
# (the documented "int registers as numbers.Number" — mamba returns
# False — int not registered against numbers ABC), `isinstance
# (1.5, numbers.Real)` (the documented "float registers as numbers.
# Real" — mamba returns False), `isinstance(1, numbers.Integral)`
# (the documented "int registers as numbers.Integral" — mamba
# returns False), `Color.RED.value == 1` (the documented "Enum
# member .value returns the wrapped value" — mamba returns None —
# .value attribute is None), `Color.RED.name == 'RED'` (the
# documented "Enum member .name returns the member name string" —
# mamba returns None), `type(Color.RED).__name__` (the documented
# "Enum member's type is the Enum class itself" — mamba returns
# 'int' — member is raw int), `len(Color)` (the documented "len of
# 3-member Enum is 3" — mamba returns 5 — len reports class-name
# length 'Color' instead of member count), `hasattr(enum,
# 'EnumMeta')` (the documented "enum exposes the EnumMeta
# metaclass" — mamba returns False), `hasattr(dataclasses,
# 'is_dataclass')` (the documented "dataclasses exposes the
# is_dataclass predicate" — mamba returns False), and `Pt(1, 2).x
# == 1` (the documented "dataclass init assigns positional args to
# fields" — mamba returns None — fields not initialized).
# Ten-pack pinned to atomic 290.
#
# Behavioral edges that CONFORM on mamba (abc — hasattr ABC/ABCMeta/
# abstractmethod/abstractclassmethod/abstractstaticmethod/abstract
# property/get_cache_token/update_abstractmethods. numbers —
# hasattr Number/Complex/Real/Rational/Integral + isinstance(1+2j,
# Complex) + isinstance('a', Number) False. enum — hasattr Enum/
# IntEnum/Flag/IntFlag/StrEnum/auto/unique + IntEnum int equality/
# arithmetic/cast. dataclasses — hasattr dataclass/field/fields/
# asdict/astuple) are covered in the matching pass fixture `test_
# abc_numbers_enum_dataclasses_value_ops`.
import numbers
import enum
import dataclasses


class _Color(enum.Enum):
    RED = 1
    GREEN = 2
    BLUE = 3


@dataclasses.dataclass
class _Pt:
    x: int
    y: int


_ledger: list[int] = []

# 1) isinstance(1, numbers.Number) — int registers as Number
#    (mamba: returns False — int not registered against numbers ABC)
assert isinstance(1, numbers.Number) == True; _ledger.append(1)

# 2) isinstance(1.5, numbers.Real) — float registers as Real
#    (mamba: returns False)
assert isinstance(1.5, numbers.Real) == True; _ledger.append(1)

# 3) isinstance(1, numbers.Integral) — int registers as Integral
#    (mamba: returns False)
assert isinstance(1, numbers.Integral) == True; _ledger.append(1)

# 4) Color.RED.value == 1 — wrapped value round-trip
#    (mamba: returns None — .value attribute is None)
assert _Color.RED.value == 1; _ledger.append(1)

# 5) Color.RED.name == 'RED' — member name string
#    (mamba: returns None)
assert _Color.RED.name == "RED"; _ledger.append(1)

# 6) type(Color.RED).__name__ == 'Color' — member's type is Enum class
#    (mamba: returns 'int' — member is raw int)
assert type(_Color.RED).__name__ == "_Color"; _ledger.append(1)

# 7) len(Color) == 3 — 3-member enum
#    (mamba: returns 5 — len reports class-name length instead of members)
assert len(_Color) == 3; _ledger.append(1)

# 8) hasattr(enum, 'EnumMeta') — EnumMeta metaclass
#    (mamba: returns False)
assert hasattr(enum, "EnumMeta") == True; _ledger.append(1)

# 9) hasattr(dataclasses, 'is_dataclass') — is_dataclass predicate
#    (mamba: returns False)
assert hasattr(dataclasses, "is_dataclass") == True; _ledger.append(1)

# 10) Pt(1, 2).x == 1 — dataclass init assigns positional args
#     (mamba: returns None — fields not initialized)
assert _Pt(1, 2).x == 1; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_numbers_enum_dataclasses_silent {sum(_ledger)} asserts")
