# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "test_lang_exception_hierarchy_handling_value_ops"
# subject = "cpython321.test_lang_exception_hierarchy_handling_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_lang_exception_hierarchy_handling_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_lang_exception_hierarchy_handling_value_ops: execute CPython 3.12 seed test_lang_exception_hierarchy_handling_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 326 pass conformance — exception hierarchy depth (IndexError/
# KeyError, ZeroDivisionError/FloatingPointError/OverflowError,
# UnicodeError/UnicodeDecode/Encode, FileNotFound/Permission/
# IsADirectory/NotADirectory/BlockingIO/ChildProcess/Connection
# (Reset/Aborted/Refused)/BrokenPipe + ConnectionError under OSError,
# StopIteration/StopAsyncIteration, GeneratorExit/KeyboardInterrupt
# /SystemExit under BaseException, Exception under BaseException,
# TypeError/NameError/AttributeError/ImportError/ModuleNotFound/
# RuntimeError/Recursion/NotImplemented), try/except specificity
# (FileNotFoundError as OSError, RecursionError as RuntimeError,
# ZeroDivisionError as ArithmeticError), except-tuple, finally/else
# timing, exception attrs (args/str/type/repr), bare raise re-raise,
# raise...from None (__cause__ + __suppress_context__), implicit
# __context__, custom Exception subclass with args/code/__str__,
# StopIteration.value, AssertionError with message, BaseException
# catch, raise instance identity. All asserts match between CPython
# 3.12 and mamba.

_ledger: list[int] = []

# 1) Exception hierarchy issubclass
assert issubclass(IndexError, LookupError); _ledger.append(1)
assert issubclass(KeyError, LookupError); _ledger.append(1)
assert issubclass(ZeroDivisionError, ArithmeticError); _ledger.append(1)
assert issubclass(FloatingPointError, ArithmeticError); _ledger.append(1)
assert issubclass(OverflowError, ArithmeticError); _ledger.append(1)
assert issubclass(UnicodeError, ValueError); _ledger.append(1)
assert issubclass(UnicodeDecodeError, UnicodeError); _ledger.append(1)
assert issubclass(UnicodeEncodeError, UnicodeError); _ledger.append(1)
assert issubclass(FileNotFoundError, OSError); _ledger.append(1)
assert issubclass(PermissionError, OSError); _ledger.append(1)
assert issubclass(NotADirectoryError, OSError); _ledger.append(1)
assert issubclass(IsADirectoryError, OSError); _ledger.append(1)
assert issubclass(BlockingIOError, OSError); _ledger.append(1)
assert issubclass(ChildProcessError, OSError); _ledger.append(1)
assert issubclass(BrokenPipeError, ConnectionError); _ledger.append(1)
assert issubclass(ConnectionResetError, ConnectionError); _ledger.append(1)
assert issubclass(ConnectionAbortedError, ConnectionError); _ledger.append(1)
assert issubclass(ConnectionRefusedError, ConnectionError); _ledger.append(1)
assert issubclass(ConnectionError, OSError); _ledger.append(1)
assert issubclass(StopIteration, Exception); _ledger.append(1)
assert issubclass(StopAsyncIteration, Exception); _ledger.append(1)
assert issubclass(GeneratorExit, BaseException); _ledger.append(1)
assert issubclass(KeyboardInterrupt, BaseException); _ledger.append(1)
assert issubclass(SystemExit, BaseException); _ledger.append(1)
assert issubclass(Exception, BaseException); _ledger.append(1)
assert issubclass(TypeError, Exception); _ledger.append(1)
assert issubclass(NameError, Exception); _ledger.append(1)
assert issubclass(AttributeError, Exception); _ledger.append(1)
assert issubclass(ImportError, Exception); _ledger.append(1)
assert issubclass(ModuleNotFoundError, ImportError); _ledger.append(1)
assert issubclass(RuntimeError, Exception); _ledger.append(1)
assert issubclass(RecursionError, RuntimeError); _ledger.append(1)
assert issubclass(NotImplementedError, RuntimeError); _ledger.append(1)

# 2) try/except specificity (subclass caught by base except)
_caught1 = None
try:
    raise FileNotFoundError("a")
except OSError as e:
    _caught1 = type(e).__name__
assert _caught1 == "FileNotFoundError"; _ledger.append(1)

_caught2 = None
try:
    raise RecursionError("deep")
except RuntimeError as e:
    _caught2 = type(e).__name__
assert _caught2 == "RecursionError"; _ledger.append(1)

_caught3 = None
try:
    raise ZeroDivisionError("z")
except ArithmeticError as e:
    _caught3 = type(e).__name__
assert _caught3 == "ZeroDivisionError"; _ledger.append(1)

# 3) except tuple
_caught4 = None
try:
    raise ValueError("v")
except (KeyError, ValueError, IndexError) as e:
    _caught4 = type(e).__name__
assert _caught4 == "ValueError"; _ledger.append(1)

# 4) finally / else timing
_ran_else = False
_ran_finally = False
try:
    _x = 1
except Exception:
    pass
else:
    _ran_else = True
finally:
    _ran_finally = True
assert _ran_else == True; _ledger.append(1)
assert _ran_finally == True; _ledger.append(1)

_ran_else2 = False
_ran_finally2 = False
try:
    raise ValueError("y")
except ValueError:
    pass
else:
    _ran_else2 = True
finally:
    _ran_finally2 = True
assert _ran_else2 == False; _ledger.append(1)
assert _ran_finally2 == True; _ledger.append(1)

# 5) exception attrs
try:
    raise ValueError("msg", "extra", "more")
except ValueError as e:
    assert e.args == ("msg", "extra", "more"); _ledger.append(1)
    assert str(e) == "('msg', 'extra', 'more')"; _ledger.append(1)
    assert type(e).__name__ == "ValueError"; _ledger.append(1)
    assert repr(e) == "ValueError('msg', 'extra', 'more')"; _ledger.append(1)

try:
    raise KeyError("nope")
except KeyError as e:
    assert e.args == ("nope",); _ledger.append(1)
    assert str(e) == "'nope'"; _ledger.append(1)

# 6) Exception with no args
try:
    raise Exception
except Exception as e:
    assert e.args == (); _ledger.append(1)
    assert str(e) == ""; _ledger.append(1)

# 7) bare raise re-raise
def _reraise():
    try:
        raise ValueError("orig")
    except ValueError:
        raise

_caught5 = None
try:
    _reraise()
except ValueError as e:
    _caught5 = str(e)
assert _caught5 == "orig"; _ledger.append(1)

# 8) raise from None (__cause__ + __suppress_context__)
_e_outer = None
try:
    try:
        raise KeyError("inner")
    except KeyError:
        raise ValueError("outer") from None
except ValueError as e:
    _e_outer = e
assert _e_outer.__cause__ is None; _ledger.append(1)
assert _e_outer.__suppress_context__ == True; _ledger.append(1)

# 9) implicit __context__
_e_ctx = None
try:
    try:
        raise ValueError("inner_ctx")
    except ValueError:
        raise RuntimeError("outer_ctx")
except RuntimeError as e:
    _e_ctx = e
assert type(_e_ctx.__context__).__name__ == "ValueError"; _ledger.append(1)
assert str(_e_ctx.__context__) == "inner_ctx"; _ledger.append(1)

# 10) custom Exception subclass
class _MyError(Exception):
    def __init__(self, code, msg):
        super().__init__(msg)
        self.code = code

try:
    raise _MyError(42, "bad")
except _MyError as e:
    assert e.code == 42; _ledger.append(1)
    assert e.args == ("bad",); _ledger.append(1)
    assert str(e) == "bad"; _ledger.append(1)
    assert isinstance(e, Exception); _ledger.append(1)

# 11) custom Exception with __str__ override
class _CE(Exception):
    def __str__(self):
        return "custom-str"
try:
    raise _CE("argy")
except _CE as e:
    assert str(e) == "custom-str"; _ledger.append(1)
    assert e.args == ("argy",); _ledger.append(1)

# 12) StopIteration.value
def _gen():
    yield 1
    return "result"

_g = _gen()
next(_g)
try:
    next(_g)
except StopIteration as e:
    assert e.value == "result"; _ledger.append(1)

# 13) AssertionError with message
try:
    assert False, "the message"
except AssertionError as e:
    assert e.args == ("the message",); _ledger.append(1)
    assert str(e) == "the message"; _ledger.append(1)

# 14) BaseException catch
_caught6 = None
try:
    raise ValueError("vv")
except BaseException as e:
    _caught6 = type(e).__name__
assert _caught6 == "ValueError"; _ledger.append(1)

# 15) raise instance identity
_inst = ValueError("preconstructed")
_caught_inst = None
try:
    raise _inst
except ValueError as e:
    _caught_inst = e
assert _caught_inst is _inst; _ledger.append(1)

# 16) Exception() with no args + raise (class form)
try:
    raise ValueError
except ValueError as e:
    assert e.args == (); _ledger.append(1)
    assert str(e) == ""; _ledger.append(1)

# 17) isinstance + Exception class
assert isinstance(ValueError("x"), ValueError); _ledger.append(1)
assert isinstance(ValueError("x"), Exception); _ledger.append(1)
assert isinstance(ValueError("x"), BaseException); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_lang_exception_hierarchy_handling_value_ops {sum(_ledger)} asserts")
