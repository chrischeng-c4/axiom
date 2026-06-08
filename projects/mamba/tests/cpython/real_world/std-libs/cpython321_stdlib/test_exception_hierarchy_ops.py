# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_exception_hierarchy_ops"
# subject = "cpython321.test_exception_hierarchy_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_exception_hierarchy_ops.py"
# status = "filled"
# ///
"""cpython321.test_exception_hierarchy_ops: execute CPython 3.12 seed test_exception_hierarchy_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the standard exception class
# hierarchy via issubclass.
# Surface: ValueError/TypeError/KeyError/IndexError/LookupError/
# ZeroDivisionError/ArithmeticError/StopIteration/NotImplementedError/
# RuntimeError/FileNotFoundError/OSError relationships to Exception
# and their immediate ancestor classes. Also asserts ValueError is
# actually raised by int() on a non-numeric string and caught by an
# `except ValueError` clause.
_ledger: list[int] = []

# All standard exception classes descend from Exception
assert issubclass(ValueError, Exception); _ledger.append(1)
assert issubclass(TypeError, Exception); _ledger.append(1)
assert issubclass(LookupError, Exception); _ledger.append(1)
assert issubclass(ArithmeticError, Exception); _ledger.append(1)
assert issubclass(StopIteration, Exception); _ledger.append(1)
assert issubclass(RuntimeError, Exception); _ledger.append(1)
assert issubclass(OSError, Exception); _ledger.append(1)

# KeyError and IndexError both inherit from LookupError
assert issubclass(KeyError, LookupError); _ledger.append(1)
assert issubclass(IndexError, LookupError); _ledger.append(1)

# ZeroDivisionError inherits from ArithmeticError
assert issubclass(ZeroDivisionError, ArithmeticError); _ledger.append(1)

# NotImplementedError inherits from RuntimeError (and Exception)
assert issubclass(NotImplementedError, RuntimeError); _ledger.append(1)
assert issubclass(NotImplementedError, Exception); _ledger.append(1)

# FileNotFoundError inherits from OSError
assert issubclass(FileNotFoundError, OSError); _ledger.append(1)

# A class is always a subclass of itself
assert issubclass(ValueError, ValueError); _ledger.append(1)

# Cross-tree negative: TypeError is NOT a LookupError
assert not issubclass(TypeError, LookupError); _ledger.append(1)
# Cross-tree negative: KeyError is NOT an ArithmeticError
assert not issubclass(KeyError, ArithmeticError); _ledger.append(1)


# A user-defined subclass of Exception participates in the hierarchy
class _MyError(Exception):
    pass

assert issubclass(_MyError, Exception); _ledger.append(1)
# An instance of MyError is also an instance of Exception
assert isinstance(_MyError("x"), Exception); _ledger.append(1)


# int() on a non-numeric string raises ValueError, which is catchable
def _trip(s):
    try:
        return int(s)
    except ValueError:
        return -1

# Numeric input round-trips through int(), non-numeric trips the
# except branch; bind to local first to dodge int-identity quirk.
ok = _trip("42")
assert ok - 42 == 0; _ledger.append(1)
bad = _trip("nope")
assert bad - (-1) == 0; _ledger.append(1)


# Raising and catching a custom Exception subclass preserves the
# message text under str(e)
try:
    raise _MyError("custom-message")
    _ledger.append(0)
except _MyError as e:
    assert str(e) == "custom-message"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_exception_hierarchy_ops {sum(_ledger)} asserts")
