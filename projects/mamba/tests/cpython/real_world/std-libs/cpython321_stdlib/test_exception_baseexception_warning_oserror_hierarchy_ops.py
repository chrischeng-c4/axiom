# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_exception_baseexception_warning_oserror_hierarchy_ops"
# subject = "cpython321.test_exception_baseexception_warning_oserror_hierarchy_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_exception_baseexception_warning_oserror_hierarchy_ops.py"
# status = "filled"
# ///
"""cpython321.test_exception_baseexception_warning_oserror_hierarchy_ops: execute CPython 3.12 seed test_exception_baseexception_warning_oserror_hierarchy_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the built-in EXCEPTION HIERARCHY
# surface — the matching subset that the existing
# `test_exception_hierarchy_ops.py` does NOT already exercise. That
# file covers the canonical `Exception` / `LookupError` /
# `ArithmeticError` / `RuntimeError` / `OSError` core (ValueError /
# TypeError / LookupError / KeyError / IndexError / ZeroDivisionError
# / NotImplementedError / FileNotFoundError → bases). This seed fills
# the orthogonal three sub-trees:
#   1. `BaseException` root — every non-system exception is a
#      `BaseException`, plus the canonical system-exit family
#      (`SystemExit`, `KeyboardInterrupt`, `GeneratorExit`) which is
#      a `BaseException` but NOT an `Exception`;
#   2. `Warning` family — `Warning` itself plus the canonical
#      `DeprecationWarning`, `UserWarning`, `RuntimeWarning`,
#      `FutureWarning` subclasses;
#   3. `OSError` extended family — `ConnectionError` and
#      `ConnectionAbortedError`, `PermissionError`,
#      `IsADirectoryError`, `NotADirectoryError`, `TimeoutError`,
#      `BlockingIOError` (the IOError-family that all subclass
#      `OSError`);
#   4. `UnicodeError` family — `UnicodeError` →`ValueError` plus
#      `UnicodeEncodeError`, `UnicodeDecodeError`;
#   5. miscellaneous — `AssertionError`, `NameError`, `MemoryError`,
#      `OverflowError` → `ArithmeticError`, `RecursionError` →
#      `RuntimeError`, `ImportError` + `ModuleNotFoundError`,
#      `SyntaxError`, `StopAsyncIteration`, `AttributeError`,
#      `StopIteration`.
#
# Surface (the matching subset between mamba and CPython):
#   • Every non-system exception subclass of `BaseException`;
#   • SystemExit / KeyboardInterrupt / GeneratorExit subclass
#     BaseException directly (and are NOT subclasses of Exception);
#   • Warning subclass of Exception;
#   • DeprecationWarning / UserWarning / RuntimeWarning /
#     FutureWarning subclass of Warning;
#   • OSError extended family — Connection/Permission/IsADirectory/
#     NotADirectory/Timeout/BlockingIO all subclass OSError;
#   • ConnectionAbortedError → ConnectionError → OSError chain;
#   • UnicodeError → ValueError; UnicodeEncodeError /
#     UnicodeDecodeError → UnicodeError;
#   • OverflowError → ArithmeticError; RecursionError → RuntimeError;
#   • ImportError → Exception; ModuleNotFoundError → ImportError;
#   • SyntaxError → Exception;
#   • Exception instance .args attribute discipline — args is a tuple
#     of constructor positional args;
#   • isinstance discipline — an instance of a subclass is also an
#     instance of the parent class;
#   • raise / except chain — catching the parent catches any subclass.
_ledger: list[int] = []

# BaseException root
assert issubclass(Exception, BaseException); _ledger.append(1)
assert issubclass(ValueError, BaseException); _ledger.append(1)
assert issubclass(TypeError, BaseException); _ledger.append(1)
assert issubclass(KeyError, BaseException); _ledger.append(1)
assert issubclass(IndexError, BaseException); _ledger.append(1)
assert issubclass(AttributeError, BaseException); _ledger.append(1)
assert issubclass(OSError, BaseException); _ledger.append(1)
assert issubclass(LookupError, BaseException); _ledger.append(1)
assert issubclass(ArithmeticError, BaseException); _ledger.append(1)
assert issubclass(ZeroDivisionError, BaseException); _ledger.append(1)

# SystemExit / KeyboardInterrupt / GeneratorExit subclass BaseException
assert issubclass(SystemExit, BaseException); _ledger.append(1)
assert issubclass(KeyboardInterrupt, BaseException); _ledger.append(1)
assert issubclass(GeneratorExit, BaseException); _ledger.append(1)

# Warning subclass of Exception
assert issubclass(Warning, Exception); _ledger.append(1)
assert issubclass(Warning, BaseException); _ledger.append(1)

# Warning subclasses
assert issubclass(DeprecationWarning, Warning); _ledger.append(1)
assert issubclass(UserWarning, Warning); _ledger.append(1)
assert issubclass(RuntimeWarning, Warning); _ledger.append(1)
assert issubclass(FutureWarning, Warning); _ledger.append(1)
assert issubclass(DeprecationWarning, Exception); _ledger.append(1)
assert issubclass(UserWarning, Exception); _ledger.append(1)
assert issubclass(RuntimeWarning, Exception); _ledger.append(1)
assert issubclass(FutureWarning, Exception); _ledger.append(1)

# OSError extended family
assert issubclass(ConnectionError, OSError); _ledger.append(1)
assert issubclass(ConnectionAbortedError, ConnectionError); _ledger.append(1)
assert issubclass(ConnectionAbortedError, OSError); _ledger.append(1)
assert issubclass(PermissionError, OSError); _ledger.append(1)
assert issubclass(IsADirectoryError, OSError); _ledger.append(1)
assert issubclass(NotADirectoryError, OSError); _ledger.append(1)
assert issubclass(TimeoutError, OSError); _ledger.append(1)
assert issubclass(BlockingIOError, OSError); _ledger.append(1)

# OSError extended → Exception (transitive)
assert issubclass(ConnectionError, Exception); _ledger.append(1)
assert issubclass(PermissionError, Exception); _ledger.append(1)
assert issubclass(TimeoutError, Exception); _ledger.append(1)

# UnicodeError family
assert issubclass(UnicodeError, ValueError); _ledger.append(1)
assert issubclass(UnicodeError, Exception); _ledger.append(1)
assert issubclass(UnicodeEncodeError, UnicodeError); _ledger.append(1)
assert issubclass(UnicodeDecodeError, UnicodeError); _ledger.append(1)
assert issubclass(UnicodeEncodeError, ValueError); _ledger.append(1)
assert issubclass(UnicodeDecodeError, ValueError); _ledger.append(1)

# Miscellaneous
assert issubclass(AssertionError, Exception); _ledger.append(1)
assert issubclass(AssertionError, BaseException); _ledger.append(1)
assert issubclass(NameError, Exception); _ledger.append(1)
assert issubclass(MemoryError, Exception); _ledger.append(1)
assert issubclass(OverflowError, ArithmeticError); _ledger.append(1)
assert issubclass(OverflowError, Exception); _ledger.append(1)
assert issubclass(RecursionError, RuntimeError); _ledger.append(1)
assert issubclass(RecursionError, Exception); _ledger.append(1)
assert issubclass(ImportError, Exception); _ledger.append(1)
assert issubclass(ModuleNotFoundError, ImportError); _ledger.append(1)
assert issubclass(ModuleNotFoundError, Exception); _ledger.append(1)
assert issubclass(SyntaxError, Exception); _ledger.append(1)
assert issubclass(StopAsyncIteration, Exception); _ledger.append(1)
assert issubclass(StopIteration, Exception); _ledger.append(1)
assert issubclass(AttributeError, Exception); _ledger.append(1)

# Disjoint hierarchy — cross-tree non-subclass
assert not issubclass(ValueError, TypeError); _ledger.append(1)
assert not issubclass(KeyError, ValueError); _ledger.append(1)
assert not issubclass(OSError, ValueError); _ledger.append(1)
assert not issubclass(Warning, ValueError); _ledger.append(1)

# Exception instance .args discipline
assert ValueError().args == (); _ledger.append(1)
assert ValueError("a").args == ("a",); _ledger.append(1)
assert ValueError("a", "b").args == ("a", "b"); _ledger.append(1)
assert KeyError("k").args == ("k",); _ledger.append(1)
assert Exception().args == (); _ledger.append(1)
assert Exception("msg").args == ("msg",); _ledger.append(1)
assert RuntimeError("x").args == ("x",); _ledger.append(1)
assert TypeError().args == (); _ledger.append(1)
assert AttributeError("obj").args == ("obj",); _ledger.append(1)

# isinstance discipline — instance is_a subclass
assert isinstance(ValueError("a"), Exception); _ledger.append(1)
assert isinstance(ValueError("a"), BaseException); _ledger.append(1)
assert isinstance(TypeError("t"), Exception); _ledger.append(1)
assert isinstance(TypeError("t"), BaseException); _ledger.append(1)
assert isinstance(KeyError("k"), LookupError); _ledger.append(1)
assert isinstance(KeyError("k"), Exception); _ledger.append(1)
assert isinstance(IndexError("i"), LookupError); _ledger.append(1)
assert isinstance(OverflowError("o"), ArithmeticError); _ledger.append(1)
assert isinstance(RecursionError("r"), RuntimeError); _ledger.append(1)
assert isinstance(AssertionError("a"), Exception); _ledger.append(1)

# raise / except — catching the parent catches subclass
def _t1():
    try:
        raise KeyError("k")
    except LookupError:
        return "caught LookupError"
assert _t1() == "caught LookupError"; _ledger.append(1)

def _t2():
    try:
        raise IndexError("i")
    except Exception:
        return "caught Exception"
assert _t2() == "caught Exception"; _ledger.append(1)

def _t3():
    try:
        raise ZeroDivisionError("z")
    except ArithmeticError:
        return "caught ArithmeticError"
assert _t3() == "caught ArithmeticError"; _ledger.append(1)

def _t4():
    try:
        raise OverflowError("o")
    except ArithmeticError:
        return "caught ArithmeticError"
assert _t4() == "caught ArithmeticError"; _ledger.append(1)

def _t5():
    try:
        raise ValueError("v")
    except BaseException:
        return "caught BaseException"
assert _t5() == "caught BaseException"; _ledger.append(1)

def _t6():
    try:
        raise AssertionError("a")
    except Exception:
        return "caught Exception"
assert _t6() == "caught Exception"; _ledger.append(1)

# str() of exception
assert str(ValueError("msg")) == "msg"; _ledger.append(1)
assert str(TypeError("t")) == "t"; _ledger.append(1)
assert str(RuntimeError("r")) == "r"; _ledger.append(1)
assert str(Exception("e")) == "e"; _ledger.append(1)

# Reflexive — every class is subclass of itself
assert issubclass(ValueError, ValueError); _ledger.append(1)
assert issubclass(BaseException, BaseException); _ledger.append(1)
assert issubclass(Exception, Exception); _ledger.append(1)
assert issubclass(OSError, OSError); _ledger.append(1)
assert issubclass(Warning, Warning); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_exception_baseexception_warning_oserror_hierarchy_ops {sum(_ledger)} asserts")
