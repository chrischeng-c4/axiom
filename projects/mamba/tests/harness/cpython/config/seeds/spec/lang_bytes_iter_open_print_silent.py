# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `b"hello".upper()` / `b"HELLO".lower()`
# (the documented "bytes objects expose .upper/.lower returning a new
# bytes with ASCII case folded" surface — mamba raises AttributeError
# claiming the methods don't exist on `bytes`), `type(iter([])).__name__
# == "list_iterator"` / `type(iter("")).__name__ == "str_ascii_iterator"`
# (the documented "iter() returns an iterator of a built-in iterator
# type whose typename identifies the source sequence kind" — mamba's
# iterators are boxed-handle ints, so `type(...).__name__` returns
# 'int' and the typename discriminator is lost), `open(path, "wb")
# .write(b"...")` (the documented "open() in binary write mode returns
# a writable binary file object" — mamba returns None for `wb` mode so
# `.write` raises AttributeError), `tempfile.mkstemp()` result shape
# (the documented "mkstemp returns a (fd, path) tuple of ints/str" —
# mamba returns a dict-like, breaking `os.close(fd)` consumers),
# `print(..., file=buf)` writing to `buf` (the documented "the `file`
# keyword redirects output to the given write-supporting object" —
# mamba ignores `file=` and writes to stdout, leaving `buf.getvalue()`
# empty), and `r.__cause__ is not None` after `raise ... from ...` (the
# documented "the `is`/`is not` operators evaluate to bool True/False"
# — mamba returns int 1 so `repr(...)` is '1' rather than 'True').
# Ten-pack pinned to atomic 250.
#
# Behavioral edges that CONFORM on mamba (exception class hierarchy via
# issubclass — BaseException/Exception/ValueError/KeyError/IndexError/
# LookupError/FileNotFoundError/PermissionError/IsADirectoryError/
# OSError/ZeroDivisionError/ArithmeticError/TypeError/AttributeError/
# NameError/RuntimeError/ImportError/StopIteration/KeyboardInterrupt/
# SystemExit/Warning/MemoryError/RecursionError/UnicodeError/
# OverflowError; try/except catching exact class, catching parent
# LookupError for KeyError, try-else when no exception, try-finally
# normal completion; bytes construction/.hex()/.fromhex()/.split()/
# .join()/.replace()/.find()/.startswith()/.decode()/.count()/
# bytearray/len/index/slice/+/*/in; iter+next sequence/default,
# StopIteration on exhausted iter, basic generator -> list; text-mode
# file I/O write+read, writelines+readlines, os.path.exists; traceback
# deep hasattr extract_stack/format_stack/format_tb/format_exception/
# walk_tb/print_exc/print_stack/TracebackException/FrameSummary/
# StackSummary/clear_frames; tempfile module-level hasattr mkstemp/
# NamedTemporaryFile/TemporaryDirectory/gettempdir) are covered in the
# matching pass fixture
# `test_exceptions_bytes_iter_file_traceback_value_ops`.
import io
from typing import Any


_ledger: list[int] = []

# 1) b"hello".upper() — bytes-method surface contract
#    (mamba: AttributeError 'bytes' object has no attribute 'upper')
def _bytes_upper() -> Any:
    try:
        return b"hello".upper()
    except AttributeError:
        return None
assert _bytes_upper() == b"HELLO"; _ledger.append(1)

# 2) b"HELLO".lower() — bytes-method surface contract
#    (mamba: AttributeError 'bytes' object has no attribute 'lower')
def _bytes_lower() -> Any:
    try:
        return b"HELLO".lower()
    except AttributeError:
        return None
assert _bytes_lower() == b"hello"; _ledger.append(1)

# 3) type(iter([1, 2, 3])).__name__ — list-iterator type-name contract
#    (mamba: returns 'int' because iterators are boxed handles)
assert type(iter([1, 2, 3])).__name__ == "list_iterator"; _ledger.append(1)

# 4) type(iter("abc")).__name__ — str-iterator type-name contract
#    (mamba: returns 'int' because iterators are boxed handles)
assert type(iter("abc")).__name__ == "str_ascii_iterator"; _ledger.append(1)

# 5) open(path, "wb") -> BufferedWriter — binary-write open contract
#    (mamba: returns None so .write raises AttributeError)
_WB_PATH = "/tmp/mamba_atomic_250_spec_wb.bin"
def _open_wb_write() -> Any:
    try:
        with open(_WB_PATH, "wb") as f:
            f.write(b"\x00\x01\x02")
        with open(_WB_PATH, "rb") as f:
            return f.read()
    except AttributeError:
        return None
    except Exception:
        return "ERR"
assert _open_wb_write() == b"\x00\x01\x02"; _ledger.append(1)

# 6) tempfile.mkstemp() -> (fd, path) tuple — mkstemp result-shape contract
#    (mamba: returns a dict-like that breaks os.close consumers)
def _mkstemp_shape() -> str:
    import tempfile
    try:
        r = tempfile.mkstemp(suffix=".txt")
    except Exception:
        return "ERR"
    return type(r).__name__
assert _mkstemp_shape() == "tuple"; _ledger.append(1)

# 7) print(..., sep=, file=buf) -> writes to buf, not stdout
#    (mamba: ignores file=, writes to stdout, buf stays empty)
def _print_sep_to_buf() -> str:
    buf = io.StringIO()
    print("a", "b", "c", sep="-", file=buf)
    return buf.getvalue()
assert _print_sep_to_buf() == "a-b-c\n"; _ledger.append(1)

# 8) print(..., end=, file=buf) -> writes to buf, not stdout
#    (mamba: ignores file=, writes to stdout, buf stays empty)
def _print_end_to_buf() -> str:
    buf = io.StringIO()
    print("hi", end="!", file=buf)
    return buf.getvalue()
assert _print_end_to_buf() == "hi!"; _ledger.append(1)

# 9) print(..., flush=, file=buf) -> writes to buf, not stdout
#    (mamba: ignores file=, writes to stdout, buf stays empty)
def _print_flush_to_buf() -> str:
    buf = io.StringIO()
    print("hi", file=buf, flush=True)
    return buf.getvalue()
assert _print_flush_to_buf() == "hi\n"; _ledger.append(1)

# 10) `raise ... from ...` then `r.__cause__ is not None` -> bool True
#     (mamba: `is not None` returns int 1, so repr is '1' not 'True')
def _raise_from_cause_repr() -> str:
    try:
        try:
            raise ValueError("inner")
        except ValueError as e:
            raise RuntimeError("outer") from e
    except RuntimeError as r:
        return repr(r.__cause__ is not None)
assert _raise_from_cause_repr() == "True"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_bytes_iter_open_print_silent {sum(_ledger)} asserts")
