# Operational AssertionPass seed for SILENT divergences across the
# threading.Lock class-identity + threading.Lock.acquire/release
# instance-method surface + threading.Event instance-method surface
# + socket extended module helpers + select full module hasattr
# surface + io extended module helpers + io.StringIO/BytesIO
# class-identity + io.StringIO/BytesIO write+getvalue value
# contract + csv.reader iterable value contract pinned by atomic
# 182: `threading` (the documented `type(threading.Lock()).__name__
# == "lock"` class-identity contract + the documented
# `Lock.acquire()` / `Lock.release()` instance-method contract +
# the documented `Event.is_set()` / `Event.set()` / `Event.clear()`
# instance-method contract), `socket` (the documented `gaierror` /
# `timeout` extended class identifiers), `select` (the documented
# `select` / `poll` / `kqueue` / `PIPE_BUF` extended class /
# function identifiers), `io` (the documented `open` / `FileIO` /
# `TextIOWrapper` / `BufferedReader` / `BufferedWriter` /
# `DEFAULT_BUFFER_SIZE` extended class / function / sentinel
# identifiers + the documented `type(io.StringIO()).__name__ ==
# "StringIO"` / `type(io.BytesIO()).__name__ == "BytesIO"`
# class-identity contract + the documented StringIO / BytesIO
# write+getvalue value contract), and `csv` (the documented
# `list(csv.reader(stream))` iterable-row-emission contract).
#
# The matching subset (full threading module hasattr surface +
# partial socket module hasattr surface (socket / AF_INET /
# AF_INET6 / AF_UNIX / SOCK_STREAM / SOCK_DGRAM / gethostname /
# gethostbyname / create_connection / create_server) + socket
# integer-constant value contract + socket.gethostname() str
# return type + full csv module hasattr surface + partial io
# module hasattr surface (StringIO / BytesIO)) is covered by
# `test_threading_socket_csv_io_hasattr_ops`; this fixture pins
# the CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • type(threading.Lock()).__name__ == "lock" — documented
#     class-identity contract (mamba: returns "Lock" — the
#     casing on the class identifier diverges);
#   • threading.Lock().acquire() works — documented mutating
#     helper (mamba: raises AttributeError);
#   • threading.Lock().release() works — documented mutating
#     helper (mamba: raises AttributeError);
#   • threading.Event().is_set() == False — documented
#     instance-method (mamba: raises AttributeError);
#   • threading.Event().set() + .is_set() == True —
#     documented mutating helper (mamba: raises
#     AttributeError);
#   • threading.Event().clear() + .is_set() == False —
#     documented mutating helper (mamba: raises
#     AttributeError);
#   • hasattr(socket, "gaierror") is True — documented
#     class identifier (mamba: False);
#   • hasattr(socket, "timeout") is True — documented
#     class identifier (mamba: False);
#   • hasattr(select, "select") is True — documented
#     function identifier (mamba: False);
#   • hasattr(select, "poll") is True — documented function
#     identifier (mamba: False);
#   • hasattr(select, "kqueue") is True — documented class
#     identifier (mamba: False);
#   • hasattr(select, "PIPE_BUF") is True — documented
#     integer sentinel (mamba: False);
#   • hasattr(io, "open") is True — documented function
#     identifier (mamba: False);
#   • hasattr(io, "FileIO") is True — documented class
#     identifier (mamba: False);
#   • hasattr(io, "TextIOWrapper") is True — documented
#     class identifier (mamba: False);
#   • hasattr(io, "BufferedReader") is True — documented
#     class identifier (mamba: False);
#   • hasattr(io, "BufferedWriter") is True — documented
#     class identifier (mamba: False);
#   • hasattr(io, "DEFAULT_BUFFER_SIZE") is True —
#     documented integer sentinel (mamba: False);
#   • type(io.StringIO()).__name__ == "StringIO" —
#     documented class-identity contract (mamba: returns
#     "dict" — the StringIO constructor returns a plain
#     dict not a StringIO instance);
#   • io.StringIO().write("hello") + .getvalue() ==
#     "hello" — documented value contract (mamba: returns
#     empty "" — the write+getvalue layer does not buffer);
#   • type(io.BytesIO()).__name__ == "BytesIO" —
#     documented class-identity contract (mamba: returns
#     "dict");
#   • io.BytesIO().write(b"abc") + .getvalue() == b"abc" —
#     documented value contract (mamba: returns empty b"");
#   • list(csv.reader(StringIO("a,b\n1,2\n"))) ==
#     [["a","b"], ["1","2"]] — documented iterable-row-
#     emission contract (mamba: returns [] — the csv
#     reader does not iterate the underlying stream).
import threading as _threading_mod
import socket as _socket_mod
import select as _select_mod
import io as _io_mod
import csv as _csv_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / instance-method / value-contract behavior
# that mamba's bundled type stubs do not surface accurately.
threading: Any = _threading_mod
socket: Any = _socket_mod
select: Any = _select_mod
io: Any = _io_mod
csv: Any = _csv_mod


_ledger: list[int] = []

# 1) threading.Lock — class-identity + acquire + release
assert type(threading.Lock()).__name__ == "lock"; _ledger.append(1)
_lk = threading.Lock()
_lk.acquire()
_lk.release()
_ledger.append(1)

# 2) threading.Event — is_set + set + clear
_ev = threading.Event()
assert _ev.is_set() == False; _ledger.append(1)
_ev.set()
assert _ev.is_set() == True; _ledger.append(1)
_ev.clear()
assert _ev.is_set() == False; _ledger.append(1)

# 3) socket — extended class identifiers
assert hasattr(socket, "gaierror") == True; _ledger.append(1)
assert hasattr(socket, "timeout") == True; _ledger.append(1)

# 4) select — full module hasattr surface
assert hasattr(select, "select") == True; _ledger.append(1)
assert hasattr(select, "poll") == True; _ledger.append(1)
assert hasattr(select, "kqueue") == True; _ledger.append(1)
assert hasattr(select, "PIPE_BUF") == True; _ledger.append(1)

# 5) io — extended class / function / sentinel identifiers
assert hasattr(io, "open") == True; _ledger.append(1)
assert hasattr(io, "FileIO") == True; _ledger.append(1)
assert hasattr(io, "TextIOWrapper") == True; _ledger.append(1)
assert hasattr(io, "BufferedReader") == True; _ledger.append(1)
assert hasattr(io, "BufferedWriter") == True; _ledger.append(1)
assert hasattr(io, "DEFAULT_BUFFER_SIZE") == True; _ledger.append(1)

# 6) io.StringIO — class-identity + write+getvalue value
assert type(io.StringIO()).__name__ == "StringIO"; _ledger.append(1)
_s = io.StringIO()
_s.write("hello")
assert _s.getvalue() == "hello"; _ledger.append(1)

# 7) io.BytesIO — class-identity + write+getvalue value
assert type(io.BytesIO()).__name__ == "BytesIO"; _ledger.append(1)
_b = io.BytesIO()
_b.write(b"abc")
assert _b.getvalue() == b"abc"; _ledger.append(1)

# 8) csv.reader — iterable-row-emission value
_stream = io.StringIO("a,b,c\n1,2,3\n4,5,6\n")
_rows = list(csv.reader(_stream))
assert _rows == [["a", "b", "c"], ["1", "2", "3"], ["4", "5", "6"]]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_threading_lock_select_io_csv_silent {sum(_ledger)} asserts")
