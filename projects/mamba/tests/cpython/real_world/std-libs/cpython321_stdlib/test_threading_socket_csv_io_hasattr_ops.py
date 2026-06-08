# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_threading_socket_csv_io_hasattr_ops"
# subject = "cpython321.test_threading_socket_csv_io_hasattr_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_threading_socket_csv_io_hasattr_ops.py"
# status = "filled"
# ///
"""cpython321.test_threading_socket_csv_io_hasattr_ops: execute CPython 3.12 seed test_threading_socket_csv_io_hasattr_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `threading` / `socket` / `csv` / `io` four-pack pinned to
# atomic 182: `threading` (the documented full module-level
# helper hasattr surface — `Thread` / `Lock` / `RLock` /
# `Event` / `Condition` / `Semaphore` / `BoundedSemaphore` /
# `Timer` / `Barrier` / `current_thread` / `active_count` /
# `enumerate` / `main_thread` / `local`), `socket` (the
# documented partial module-level helper hasattr surface —
# `socket` / `AF_INET` / `AF_INET6` / `AF_UNIX` /
# `SOCK_STREAM` / `SOCK_DGRAM` / `gethostname` /
# `gethostbyname` / `create_connection` / `create_server` +
# the documented `AF_INET` / `SOCK_STREAM` integer-constant
# value contract + the documented `gethostname()` str
# return-type contract), `csv` (the documented full module-
# level helper hasattr surface — `reader` / `writer` /
# `DictReader` / `DictWriter` / `Dialect` / `excel` /
# `QUOTE_ALL` / `QUOTE_MINIMAL` / `QUOTE_NONNUMERIC` /
# `QUOTE_NONE` / `field_size_limit` / `register_dialect` /
# `list_dialects`), and `io` (the documented partial module-
# level helper hasattr surface — `StringIO` / `BytesIO`).
#
# The matching subset between mamba and CPython is the full
# `threading` module hasattr surface (Lock.acquire / Lock.
# release / Event.is_set / Event.set / Event.clear DIVERGE +
# type(Lock()).__name__ "Lock" vs "lock" DIVERGES), the
# partial `socket` module hasattr surface
# (`gaierror` / `timeout` DIVERGE) + the documented integer-
# constant value contract + the documented `gethostname()`
# str return-type contract, the full `csv` module hasattr
# surface (the csv.reader value contract DIVERGES), and the
# partial `io` module hasattr surface (`open` / `FileIO` /
# `TextIOWrapper` / `BufferedReader` / `BufferedWriter` /
# `DEFAULT_BUFFER_SIZE` DIVERGE + the StringIO / BytesIO
# write+getvalue value contract DIVERGES).
#
# Surface in this fixture:
#   • threading — full module hasattr surface (Thread /
#     Lock / RLock / Event / Condition / Semaphore /
#     BoundedSemaphore / Timer / Barrier / current_thread /
#     active_count / enumerate / main_thread / local);
#   • socket — partial module hasattr surface (socket /
#     AF_INET / AF_INET6 / AF_UNIX / SOCK_STREAM /
#     SOCK_DGRAM / gethostname / gethostbyname /
#     create_connection / create_server);
#   • socket — AF_INET / SOCK_STREAM integer-constant
#     value contract;
#   • socket — gethostname() str return-type contract;
#   • csv — full module hasattr surface (reader / writer /
#     DictReader / DictWriter / Dialect / excel /
#     QUOTE_ALL / QUOTE_MINIMAL / QUOTE_NONNUMERIC /
#     QUOTE_NONE / field_size_limit / register_dialect /
#     list_dialects);
#   • io — partial module hasattr surface (StringIO /
#     BytesIO).
#
# Behavioral edges that DIVERGE on mamba
# (type(threading.Lock()).__name__ returns "Lock" not "lock",
# Lock.acquire / Lock.release / Event.is_set / Event.set /
# Event.clear all raise AttributeError, hasattr(socket,
# "gaierror") / "timeout" False, hasattr(select, "select") /
# "poll" / "kqueue" / "PIPE_BUF" all False, hasattr(io,
# "open") / "FileIO" / "TextIOWrapper" / "BufferedReader" /
# "BufferedWriter" / "DEFAULT_BUFFER_SIZE" False,
# type(io.StringIO()).__name__ returns "dict" not "StringIO",
# StringIO.write + getvalue returns empty "" not "hello",
# type(io.BytesIO()).__name__ returns "dict" not "BytesIO",
# BytesIO.write + getvalue returns empty b"" not b"abc",
# list(csv.reader(s)) returns [] not the documented rows)
# are covered in the matching spec fixture
# `lang_threading_lock_select_io_csv_silent`.
import threading
import socket
import csv
import io


_ledger: list[int] = []

# 1) threading — full module hasattr surface
assert hasattr(threading, "Thread") == True; _ledger.append(1)
assert hasattr(threading, "Lock") == True; _ledger.append(1)
assert hasattr(threading, "RLock") == True; _ledger.append(1)
assert hasattr(threading, "Event") == True; _ledger.append(1)
assert hasattr(threading, "Condition") == True; _ledger.append(1)
assert hasattr(threading, "Semaphore") == True; _ledger.append(1)
assert hasattr(threading, "BoundedSemaphore") == True; _ledger.append(1)
assert hasattr(threading, "Timer") == True; _ledger.append(1)
assert hasattr(threading, "Barrier") == True; _ledger.append(1)
assert hasattr(threading, "current_thread") == True; _ledger.append(1)
assert hasattr(threading, "active_count") == True; _ledger.append(1)
assert hasattr(threading, "enumerate") == True; _ledger.append(1)
assert hasattr(threading, "main_thread") == True; _ledger.append(1)
assert hasattr(threading, "local") == True; _ledger.append(1)

# 2) socket — partial module hasattr surface
#    (gaierror / timeout DIVERGE — moved to spec fixture)
assert hasattr(socket, "socket") == True; _ledger.append(1)
assert hasattr(socket, "AF_INET") == True; _ledger.append(1)
assert hasattr(socket, "AF_INET6") == True; _ledger.append(1)
assert hasattr(socket, "AF_UNIX") == True; _ledger.append(1)
assert hasattr(socket, "SOCK_STREAM") == True; _ledger.append(1)
assert hasattr(socket, "SOCK_DGRAM") == True; _ledger.append(1)
assert hasattr(socket, "gethostname") == True; _ledger.append(1)
assert hasattr(socket, "gethostbyname") == True; _ledger.append(1)
assert hasattr(socket, "create_connection") == True; _ledger.append(1)
assert hasattr(socket, "create_server") == True; _ledger.append(1)

# 3) socket — integer-constant value contract
assert socket.AF_INET == 2; _ledger.append(1)
assert socket.SOCK_STREAM == 1; _ledger.append(1)

# 4) socket — gethostname() str return-type contract
assert type(socket.gethostname()).__name__ == "str"; _ledger.append(1)

# 5) csv — full module hasattr surface
assert hasattr(csv, "reader") == True; _ledger.append(1)
assert hasattr(csv, "writer") == True; _ledger.append(1)
assert hasattr(csv, "DictReader") == True; _ledger.append(1)
assert hasattr(csv, "DictWriter") == True; _ledger.append(1)
assert hasattr(csv, "Dialect") == True; _ledger.append(1)
assert hasattr(csv, "excel") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_ALL") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_MINIMAL") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_NONNUMERIC") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_NONE") == True; _ledger.append(1)
assert hasattr(csv, "field_size_limit") == True; _ledger.append(1)
assert hasattr(csv, "register_dialect") == True; _ledger.append(1)
assert hasattr(csv, "list_dialects") == True; _ledger.append(1)

# 6) io — partial module hasattr surface
#    (open / FileIO / TextIOWrapper / BufferedReader /
#    BufferedWriter / DEFAULT_BUFFER_SIZE DIVERGE — moved
#    to spec fixture; StringIO / BytesIO write+getvalue
#    behavior also DIVERGES — moved to spec fixture)
assert hasattr(io, "StringIO") == True; _ledger.append(1)
assert hasattr(io, "BytesIO") == True; _ledger.append(1)

# NB: type(threading.Lock()).__name__ returns "Lock" on
# mamba but "lock" on CPython, Lock.acquire / Lock.release
# / Event.is_set / Event.set / Event.clear all raise
# AttributeError on mamba, hasattr(socket, "gaierror") /
# "timeout" False on mamba, hasattr(select, "select") /
# "poll" / "kqueue" / "PIPE_BUF" all False, hasattr(io,
# "open") / "FileIO" / "TextIOWrapper" / "BufferedReader"
# / "BufferedWriter" / "DEFAULT_BUFFER_SIZE" all False,
# type(io.StringIO()).__name__ returns "dict" on mamba,
# StringIO.write + getvalue returns empty "" on mamba,
# list(csv.reader(s)) returns [] on mamba — all DIVERGE
# on mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_threading_socket_csv_io_hasattr_ops {sum(_ledger)} asserts")
