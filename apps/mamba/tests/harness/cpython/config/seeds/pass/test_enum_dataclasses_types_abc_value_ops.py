# Atomic 256 pass conformance — enum module
# (hasattr surface Enum/IntEnum/Flag/IntFlag/auto/unique + member
# identity Color.RED is Color.RED, equality Color.RED == Color.RED,
# inequality Color.RED != Color.GREEN, IntEnum equality with int
# Mode.READ == 1, int conversion int(Mode.READ) == 1, IntFlag bitwise
# Flags.READ | Flags.WRITE summed-int result, IntFlag int conversion
# int(Flags.READ) == 1, IntFlag equality with int Flags.READ == 1)
# + dataclasses module (hasattr surface dataclass/field/asdict/
# astuple/fields) + types module (hasattr surface SimpleNamespace/
# MethodType/FunctionType/LambdaType/ModuleType/MappingProxyType)
# + abc module (hasattr surface ABC/ABCMeta/abstractmethod/
# abstractproperty + concrete-subclass dispatch Dog().speak() ==
# 'woof', issubclass relation issubclass(Dog, Animal) True). All
# asserts match between CPython 3.12 and mamba.
import enum
import dataclasses
import types
import abc


_ledger: list[int] = []


class Color(enum.Enum):
    RED = 1
    GREEN = 2
    BLUE = 3


class Mode(enum.IntEnum):
    READ = 1
    WRITE = 2


class Flags(enum.IntFlag):
    READ = 1
    WRITE = 2
    EXEC = 4


# 1) enum — hasattr surface
assert hasattr(enum, "Enum") == True; _ledger.append(1)
assert hasattr(enum, "IntEnum") == True; _ledger.append(1)
assert hasattr(enum, "Flag") == True; _ledger.append(1)
assert hasattr(enum, "IntFlag") == True; _ledger.append(1)
assert hasattr(enum, "auto") == True; _ledger.append(1)
assert hasattr(enum, "unique") == True; _ledger.append(1)

# 2) enum — identity / equality / inequality
assert (Color.RED is Color.RED) == True; _ledger.append(1)
assert (Color.RED == Color.RED) == True; _ledger.append(1)
assert (Color.RED != Color.GREEN) == True; _ledger.append(1)

# 3) enum — IntEnum equality with int and int() conversion
assert (Mode.READ == 1) == True; _ledger.append(1)
assert int(Mode.READ) == 1; _ledger.append(1)
assert (Mode.WRITE == 2) == True; _ledger.append(1)
assert int(Mode.WRITE) == 2; _ledger.append(1)

# 4) enum — IntFlag bitwise OR and int identity
assert int(Flags.READ) == 1; _ledger.append(1)
assert int(Flags.WRITE) == 2; _ledger.append(1)
assert int(Flags.EXEC) == 4; _ledger.append(1)
assert int(Flags.READ | Flags.WRITE) == 3; _ledger.append(1)
assert int(Flags.READ | Flags.EXEC) == 5; _ledger.append(1)
assert (Flags.READ == 1) == True; _ledger.append(1)

# 5) dataclasses — hasattr surface
assert hasattr(dataclasses, "dataclass") == True; _ledger.append(1)
assert hasattr(dataclasses, "field") == True; _ledger.append(1)
assert hasattr(dataclasses, "asdict") == True; _ledger.append(1)
assert hasattr(dataclasses, "astuple") == True; _ledger.append(1)
assert hasattr(dataclasses, "fields") == True; _ledger.append(1)

# 6) types — hasattr surface
assert hasattr(types, "SimpleNamespace") == True; _ledger.append(1)
assert hasattr(types, "MethodType") == True; _ledger.append(1)
assert hasattr(types, "FunctionType") == True; _ledger.append(1)
assert hasattr(types, "LambdaType") == True; _ledger.append(1)
assert hasattr(types, "ModuleType") == True; _ledger.append(1)
assert hasattr(types, "MappingProxyType") == True; _ledger.append(1)

# 7) abc — hasattr surface
assert hasattr(abc, "ABC") == True; _ledger.append(1)
assert hasattr(abc, "ABCMeta") == True; _ledger.append(1)
assert hasattr(abc, "abstractmethod") == True; _ledger.append(1)
assert hasattr(abc, "abstractproperty") == True; _ledger.append(1)


# 8) abc — concrete subclass dispatches abstract method
class Animal(abc.ABC):
    @abc.abstractmethod
    def speak(self) -> str:
        return ""


class Dog(Animal):
    def speak(self) -> str:
        return "woof"


assert Dog().speak() == "woof"; _ledger.append(1)
assert issubclass(Dog, Animal) == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_enum_dataclasses_types_abc_value_ops {sum(_ledger)} asserts")
