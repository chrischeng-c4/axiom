# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `Color.RED.value` (the documented
# "enum member exposes the assigned value via .value" — mamba returns
# None), `Color.RED.name` (the documented "enum member exposes its
# attribute name via .name" — mamba returns None), `Color(1)` value-
# lookup constructor (the documented "calling the enum class with a
# member value returns the matching member" — mamba returns None
# instead of Color.RED), `Color['RED']` name-lookup subscript (the
# documented "enum class supports name-based subscript" — mamba
# raises TypeError 'type object is not subscriptable'), `len(Color)`
# (the documented "len of an enum class equals member count" — mamba
# returns 5 instead of 3), `type(Color.R) is Color` (the documented
# "enum member's class is the enum class itself" — mamba returns
# False), `Point(1, 2).x` for a `@dataclass` (the documented
# "dataclass auto-generates __init__ that binds positional args to
# fields" — mamba returns None), `repr(Point(1, 2))` for a
# `@dataclass` (the documented "dataclass auto-generates __repr__
# producing 'Point(x=1, y=2)'" — mamba returns '<Point instance>'),
# `types.SimpleNamespace(a=1).a` (the documented "SimpleNamespace
# constructor accepts kwargs and exposes them as attributes" — mamba
# raises AttributeError because the symbol resolves to a dict), and
# `abc.ABC` abstract enforcement (the documented "instantiating a
# class with an unimplemented @abstractmethod raises TypeError" —
# mamba allows instantiation silently).
# Ten-pack pinned to atomic 256.
#
# Behavioral edges that CONFORM on mamba (enum — hasattr Enum/
# IntEnum/Flag/IntFlag/auto/unique, member identity Color.RED is
# Color.RED, equality Color.RED == Color.RED, inequality Color.RED
# != Color.GREEN, IntEnum eq with int Mode.READ == 1, int(Mode.READ)
# == 1, IntFlag bitwise OR Flags.READ | Flags.WRITE summed-int, int
# conversion, eq with int. dataclasses — hasattr dataclass/field/
# asdict/astuple/fields. types — hasattr SimpleNamespace/MethodType/
# FunctionType/LambdaType/ModuleType/MappingProxyType. abc — hasattr
# ABC/ABCMeta/abstractmethod/abstractproperty, concrete subclass
# dispatch Dog().speak(), issubclass(Dog, Animal)) are covered in
# the matching pass fixture
# `test_enum_dataclasses_types_abc_value_ops`.
import enum
import dataclasses
import types
import abc
from typing import Any


class Color(enum.Enum):
    RED = 1
    GREEN = 2
    BLUE = 3


@dataclasses.dataclass
class Point:
    x: int
    y: int


class Animal(abc.ABC):
    @abc.abstractmethod
    def speak(self) -> str:
        return ""


_ledger: list[int] = []

# 1) Color.RED.value must expose the assigned int
#    (mamba: returns None)
assert Color.RED.value == 1; _ledger.append(1)

# 2) Color.RED.name must expose the attribute name
#    (mamba: returns None)
assert Color.RED.name == "RED"; _ledger.append(1)

# 3) Color(1) constructor maps value back to member
#    (mamba: returns None, so .name returns None)
def _color_call() -> Any:
    return Color(1).name
assert _color_call() == "RED"; _ledger.append(1)

# 4) Color['RED'] name-lookup subscript on the enum class
#    (mamba: raises TypeError 'type object is not subscriptable')
def _color_subscript() -> Any:
    try:
        return Color["RED"].name
    except TypeError:
        return None
assert _color_subscript() == "RED"; _ledger.append(1)

# 5) len(Color) equals member count
#    (mamba: returns 5)
assert len(Color) == 3; _ledger.append(1)

# 6) type(Color.RED) is Color — member class identity
#    (mamba: returns False)
assert (type(Color.RED) is Color) == True; _ledger.append(1)

# 7) dataclass Point(1, 2).x — auto __init__ binds positional args
#    (mamba: returns None — __init__ not generated)
def _dc_x() -> Any:
    return Point(1, 2).x
assert _dc_x() == 1; _ledger.append(1)

# 8) dataclass repr produces 'Point(x=1, y=2)'
#    (mamba: returns '<Point instance>' — __repr__ not generated)
assert repr(Point(1, 2)) == "Point(x=1, y=2)"; _ledger.append(1)

# 9) types.SimpleNamespace(a=1).a exposes kwargs as attributes
#    (mamba: AttributeError because SimpleNamespace symbol is a dict)
def _ns_attr() -> Any:
    try:
        return types.SimpleNamespace(a=1).a
    except AttributeError:
        return None
assert _ns_attr() == 1; _ledger.append(1)

# 10) abc abstract enforcement — instantiating subclass with
#     unimplemented @abstractmethod must raise TypeError
#     (mamba: allows instantiation silently)
def _abc_enforces() -> str:
    try:
        Animal()  # type: ignore[abstract]
        return "did not raise"
    except TypeError:
        return "raised TypeError"
assert _abc_enforces() == "raised TypeError"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_enum_dataclasses_types_abc_silent {sum(_ledger)} asserts")
