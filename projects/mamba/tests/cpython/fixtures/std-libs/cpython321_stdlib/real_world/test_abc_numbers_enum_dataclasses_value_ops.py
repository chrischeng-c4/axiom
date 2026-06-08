# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_abc_numbers_enum_dataclasses_value_ops"
# subject = "cpython321.test_abc_numbers_enum_dataclasses_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_abc_numbers_enum_dataclasses_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_abc_numbers_enum_dataclasses_value_ops: execute CPython 3.12 seed test_abc_numbers_enum_dataclasses_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 290 pass conformance — abc module (hasattr ABC/ABCMeta/
# abstractmethod/abstractclassmethod/abstractstaticmethod/abstract
# property/get_cache_token/update_abstractmethods) + numbers module
# (hasattr Number/Complex/Real/Rational/Integral + isinstance(1+2j,
# Complex) + isinstance('a', Number) False) + enum module (hasattr
# Enum/IntEnum/Flag/IntFlag/StrEnum/auto/unique + IntEnum equality
# == 1 + IntEnum arithmetic + IntEnum int cast) + dataclasses
# module (hasattr dataclass/field/fields/asdict/astuple).
# All asserts match between CPython 3.12 and mamba.
import abc
import numbers
import enum
import dataclasses


_ledger: list[int] = []

# 1) abc — hasattr core surface
assert hasattr(abc, "ABC") == True; _ledger.append(1)
assert hasattr(abc, "ABCMeta") == True; _ledger.append(1)
assert hasattr(abc, "abstractmethod") == True; _ledger.append(1)
assert hasattr(abc, "abstractclassmethod") == True; _ledger.append(1)
assert hasattr(abc, "abstractstaticmethod") == True; _ledger.append(1)
assert hasattr(abc, "abstractproperty") == True; _ledger.append(1)
assert hasattr(abc, "get_cache_token") == True; _ledger.append(1)
assert hasattr(abc, "update_abstractmethods") == True; _ledger.append(1)

# 2) numbers — hasattr ABC hierarchy
assert hasattr(numbers, "Number") == True; _ledger.append(1)
assert hasattr(numbers, "Complex") == True; _ledger.append(1)
assert hasattr(numbers, "Real") == True; _ledger.append(1)
assert hasattr(numbers, "Rational") == True; _ledger.append(1)
assert hasattr(numbers, "Integral") == True; _ledger.append(1)

# 3) numbers — value contracts (subset that agrees)
assert isinstance(1 + 2j, numbers.Complex) == True; _ledger.append(1)
assert isinstance("a", numbers.Number) == False; _ledger.append(1)

# 4) enum — hasattr core surface
assert hasattr(enum, "Enum") == True; _ledger.append(1)
assert hasattr(enum, "IntEnum") == True; _ledger.append(1)
assert hasattr(enum, "Flag") == True; _ledger.append(1)
assert hasattr(enum, "IntFlag") == True; _ledger.append(1)
assert hasattr(enum, "StrEnum") == True; _ledger.append(1)
assert hasattr(enum, "auto") == True; _ledger.append(1)
assert hasattr(enum, "unique") == True; _ledger.append(1)


class _CInt(enum.IntEnum):
    A = 1
    B = 2


# 5) enum — IntEnum int compatibility
assert (_CInt.A == 1) == True; _ledger.append(1)
assert (_CInt.A + 1) == 2; _ledger.append(1)
assert int(_CInt.A) == 1; _ledger.append(1)

# 6) dataclasses — hasattr core surface
assert hasattr(dataclasses, "dataclass") == True; _ledger.append(1)
assert hasattr(dataclasses, "field") == True; _ledger.append(1)
assert hasattr(dataclasses, "fields") == True; _ledger.append(1)
assert hasattr(dataclasses, "asdict") == True; _ledger.append(1)
assert hasattr(dataclasses, "astuple") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_abc_numbers_enum_dataclasses_value_ops {sum(_ledger)} asserts")
