# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_dataclasses_enum_surface_value_ops"
# subject = "cpython321.test_dataclasses_enum_surface_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_dataclasses_enum_surface_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_dataclasses_enum_surface_value_ops: execute CPython 3.12 seed test_dataclasses_enum_surface_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 301 pass conformance — dataclasses module (hasattr dataclass/
# field/fields/asdict/astuple + @dataclass-decorated class instance
# type echoes class name) + enum module (hasattr Enum/IntEnum/Flag/
# IntFlag/StrEnum/auto/unique + enum member equality reflexive + enum
# member equality between distinct members False + IntEnum integer-
# coercion arithmetic).
# All asserts match between CPython 3.12 and mamba.
import dataclasses
import enum


_ledger: list[int] = []

# 1) dataclasses — hasattr core surface (conformant subset)
assert hasattr(dataclasses, "dataclass") == True; _ledger.append(1)
assert hasattr(dataclasses, "field") == True; _ledger.append(1)
assert hasattr(dataclasses, "fields") == True; _ledger.append(1)
assert hasattr(dataclasses, "asdict") == True; _ledger.append(1)
assert hasattr(dataclasses, "astuple") == True; _ledger.append(1)


@dataclasses.dataclass
class Point:
    x: int
    y: int


# 2) dataclasses — type contract for decorated class instance
assert type(Point(1, 2)).__name__ == "Point"; _ledger.append(1)

# 3) enum — hasattr core surface
assert hasattr(enum, "Enum") == True; _ledger.append(1)
assert hasattr(enum, "IntEnum") == True; _ledger.append(1)
assert hasattr(enum, "Flag") == True; _ledger.append(1)
assert hasattr(enum, "IntFlag") == True; _ledger.append(1)
assert hasattr(enum, "StrEnum") == True; _ledger.append(1)
assert hasattr(enum, "auto") == True; _ledger.append(1)
assert hasattr(enum, "unique") == True; _ledger.append(1)


class Color(enum.Enum):
    RED = 1
    BLUE = 2


# 4) enum — member equality (conformant subset)
assert (Color.RED == Color.RED) == True; _ledger.append(1)
assert (Color.RED == Color.BLUE) == False; _ledger.append(1)


class Status(enum.IntEnum):
    OK = 200
    ERR = 500


# 5) IntEnum — integer-coercion arithmetic (conformant subset)
assert Status.OK + 5 == 205; _ledger.append(1)
assert Status.ERR - 100 == 400; _ledger.append(1)
assert int(Status.OK) == 200; _ledger.append(1)
assert int(Status.ERR) == 500; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_dataclasses_enum_surface_value_ops {sum(_ledger)} asserts")
