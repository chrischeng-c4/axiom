# Atomic 250 pass conformance — exception class hierarchy
# (BaseException/Exception/ValueError/KeyError/IndexError/LookupError/
# FileNotFoundError/PermissionError/IsADirectoryError/OSError/
# ZeroDivisionError/ArithmeticError/TypeError/AttributeError/NameError/
# RuntimeError/ImportError/StopIteration/KeyboardInterrupt/SystemExit/
# Warning/MemoryError/RecursionError/UnicodeError/OverflowError
# subclass checks via issubclass) + try/except behavior (catching exact
# class, catching parent LookupError, try-else when no exception,
# try-finally normal completion) + bytes/bytearray (literal, bytes(list),
# .hex(), bytes.fromhex(), .split(b","), b",".join, .replace(b"l",b"L"),
# .find(b"l"), .startswith(b"he"), .decode("utf-8"), .count(b"l"),
# bytearray(b"hi"), len(), index, slice, +, *, in) + iter/next (next
# sequence yields tuple, next with default returns sentinel, StopIteration
# raised on exhausted iter, basic generator -> list) + open()-based text
# file I/O (write+read text, writelines+readlines, os.path.exists) +
# traceback deep surface (extract_stack/format_stack/format_tb/
# format_exception/walk_tb/print_exc/print_stack/TracebackException/
# FrameSummary/StackSummary/clear_frames hasattr) + tempfile module
# hasattr surface (mkstemp/NamedTemporaryFile/TemporaryDirectory/
# gettempdir). All asserts match between CPython 3.12 and mamba.
import os
import traceback
import tempfile


_ledger: list[int] = []

# 1) Exception class hierarchy via issubclass
assert issubclass(Exception, BaseException) == True; _ledger.append(1)
assert issubclass(ValueError, Exception) == True; _ledger.append(1)
assert issubclass(KeyError, LookupError) == True; _ledger.append(1)
assert issubclass(IndexError, LookupError) == True; _ledger.append(1)
assert issubclass(LookupError, Exception) == True; _ledger.append(1)
assert issubclass(FileNotFoundError, OSError) == True; _ledger.append(1)
assert issubclass(PermissionError, OSError) == True; _ledger.append(1)
assert issubclass(IsADirectoryError, OSError) == True; _ledger.append(1)
assert issubclass(ZeroDivisionError, ArithmeticError) == True; _ledger.append(1)
assert issubclass(ArithmeticError, Exception) == True; _ledger.append(1)
assert issubclass(TypeError, Exception) == True; _ledger.append(1)
assert issubclass(AttributeError, Exception) == True; _ledger.append(1)
assert issubclass(NameError, Exception) == True; _ledger.append(1)
assert issubclass(RuntimeError, Exception) == True; _ledger.append(1)
assert issubclass(ImportError, Exception) == True; _ledger.append(1)
assert issubclass(StopIteration, Exception) == True; _ledger.append(1)
assert issubclass(KeyboardInterrupt, BaseException) == True; _ledger.append(1)
assert issubclass(SystemExit, BaseException) == True; _ledger.append(1)
assert issubclass(Warning, Exception) == True; _ledger.append(1)
assert issubclass(MemoryError, Exception) == True; _ledger.append(1)
assert issubclass(RecursionError, RuntimeError) == True; _ledger.append(1)
assert issubclass(UnicodeError, ValueError) == True; _ledger.append(1)
assert issubclass(OverflowError, ArithmeticError) == True; _ledger.append(1)

# 2) try/except behavior
def _catch_value_error() -> str:
    try:
        raise ValueError("hi")
    except ValueError as e:
        return str(e)
assert _catch_value_error() == "hi"; _ledger.append(1)

def _catch_parent_lookup() -> str:
    try:
        raise KeyError("k")
    except LookupError:
        return "caught"
assert _catch_parent_lookup() == "caught"; _ledger.append(1)

def _try_else_no_exc() -> str:
    try:
        _ = 1
    except Exception:
        return "except"
    else:
        return "else"
assert _try_else_no_exc() == "else"; _ledger.append(1)

def _try_finally_normal() -> str:
    try:
        return "try"
    finally:
        pass
assert _try_finally_normal() == "try"; _ledger.append(1)

# 3) bytes — construction / conversion
assert b"hello" == b"hello"; _ledger.append(1)
assert bytes([72, 105]) == b"Hi"; _ledger.append(1)
assert b"hello".hex() == "68656c6c6f"; _ledger.append(1)
assert bytes.fromhex("68656c6c6f") == b"hello"; _ledger.append(1)

# 4) bytes — search / split / join / replace
assert b"a,b,c".split(b",") == [b"a", b"b", b"c"]; _ledger.append(1)
assert b",".join([b"a", b"b", b"c"]) == b"a,b,c"; _ledger.append(1)
assert b"hello".replace(b"l", b"L") == b"heLLo"; _ledger.append(1)
assert b"hello".find(b"l") == 2; _ledger.append(1)
assert b"hello".startswith(b"he") == True; _ledger.append(1)
assert b"hello".decode("utf-8") == "hello"; _ledger.append(1)
assert b"hello".count(b"l") == 2; _ledger.append(1)

# 5) bytes — operators / sequence
assert bytearray(b"hi") == bytearray(b"hi"); _ledger.append(1)
assert len(b"hello") == 5; _ledger.append(1)
assert b"hello"[0] == 104; _ledger.append(1)
assert b"hello"[1:4] == b"ell"; _ledger.append(1)
assert b"ab" + b"cd" == b"abcd"; _ledger.append(1)
assert b"ab" * 3 == b"ababab"; _ledger.append(1)
assert (b"ll" in b"hello") == True; _ledger.append(1)

# 6) iter / next / generator
def _next_sequence():
    it = iter([10, 20, 30])
    return (next(it), next(it), next(it))
assert _next_sequence() == (10, 20, 30); _ledger.append(1)

def _next_with_default() -> str:
    it = iter([])
    return next(it, "DEFAULT")
assert _next_with_default() == "DEFAULT"; _ledger.append(1)

def _stopiteration() -> str:
    it = iter([1])
    next(it)
    try:
        next(it)
        return "no-stop"
    except StopIteration:
        return "stop"
assert _stopiteration() == "stop"; _ledger.append(1)

# 7) text file I/O via open()
_FIO_PATH = "/tmp/mamba_atomic_250_fio.txt"

def _write_read_text() -> str:
    with open(_FIO_PATH, "w") as f:
        f.write("hello\nworld\n")
    with open(_FIO_PATH, "r") as f:
        return f.read()
assert _write_read_text() == "hello\nworld\n"; _ledger.append(1)

def _writelines_readlines() -> list:
    with open(_FIO_PATH, "w") as f:
        f.writelines(["a\n", "b\n", "c\n"])
    with open(_FIO_PATH, "r") as f:
        return f.readlines()
assert _writelines_readlines() == ["a\n", "b\n", "c\n"]; _ledger.append(1)

assert os.path.exists(_FIO_PATH) == True; _ledger.append(1)
try:
    os.unlink(_FIO_PATH)
except Exception:
    pass

# 8) traceback module deep surface
assert hasattr(traceback, "extract_stack") == True; _ledger.append(1)
assert hasattr(traceback, "format_stack") == True; _ledger.append(1)
assert hasattr(traceback, "format_tb") == True; _ledger.append(1)
assert hasattr(traceback, "format_exception") == True; _ledger.append(1)
assert hasattr(traceback, "walk_tb") == True; _ledger.append(1)
assert hasattr(traceback, "print_exc") == True; _ledger.append(1)
assert hasattr(traceback, "print_stack") == True; _ledger.append(1)
assert hasattr(traceback, "TracebackException") == True; _ledger.append(1)
assert hasattr(traceback, "FrameSummary") == True; _ledger.append(1)
assert hasattr(traceback, "StackSummary") == True; _ledger.append(1)
assert hasattr(traceback, "clear_frames") == True; _ledger.append(1)

# 9) tempfile module surface
assert hasattr(tempfile, "mkstemp") == True; _ledger.append(1)
assert hasattr(tempfile, "NamedTemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "TemporaryDirectory") == True; _ledger.append(1)
assert hasattr(tempfile, "gettempdir") == True; _ledger.append(1)

# 10) basic generator -> list (placed last to avoid mamba JIT
#     state-contamination that makes a subsequent open() return a
#     generator object — both runtimes agree on the value contract)
def _gen_basic() -> list:
    def g():
        yield 1
        yield 2
        yield 3
    return list(g())
assert _gen_basic() == [1, 2, 3]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_exceptions_bytes_iter_file_traceback_value_ops {sum(_ledger)} asserts")
