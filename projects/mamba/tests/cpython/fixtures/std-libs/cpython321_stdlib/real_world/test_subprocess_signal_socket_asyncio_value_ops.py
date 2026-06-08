# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_subprocess_signal_socket_asyncio_value_ops"
# subject = "cpython321.test_subprocess_signal_socket_asyncio_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_subprocess_signal_socket_asyncio_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_subprocess_signal_socket_asyncio_value_ops: execute CPython 3.12 seed test_subprocess_signal_socket_asyncio_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of six
# bootstrap stdlib modules used by every subprocess-spawn /
# POSIX-signal / socket-constant / IO-selector / coroutine /
# context-variable path: `subprocess` (the documented `run`
# `capture_output=True` / `text=True` value contract +
# `returncode` / `stdout` attribute access + the documented
# `check_output` text-mode return + the documented `run` /
# `Popen` / `PIPE` / `STDOUT` / `DEVNULL` / `CalledProcessError`
# / `TimeoutExpired` / `check_output` / `check_call` module
# hasattr surface), `signal` (the documented `SIGINT` /
# `SIGTERM` / `SIGKILL` / `SIGUSR1` / `SIGUSR2` / `SIGHUP` /
# `SIGPIPE` / `SIGCHLD` / `SIGALRM` integer-value constant
# surface + the documented `SIG_DFL` / `SIG_IGN` / `signal` /
# `getsignal` module hasattr surface), `socket` (the documented
# `AF_INET` / `AF_INET6` / `AF_UNIX` / `SOCK_STREAM` /
# `SOCK_DGRAM` integer-value constant surface + the documented
# `gethostname` / `gethostbyname` module hasattr surface +
# `gethostname()` str-return contract), `selectors` (the
# documented `DefaultSelector` / `EVENT_READ` / `EVENT_WRITE` /
# `BaseSelector` / `PollSelector` module hasattr surface),
# `asyncio` (the documented `run` / `sleep` / `gather` module
# hasattr surface + the documented `run(coro)` await-to-return-
# value contract), and `contextvars` (the documented
# `ContextVar` / `Context` / `copy_context` module hasattr
# surface).
#
# The matching subset between mamba and CPython is the
# subprocess run / check_output text-mode + hasattr surface
# layer, the signal int-constant + hasattr surface layer, the
# socket int-constant + hasattr surface + gethostname str-
# return layer, the selectors module hasattr surface layer,
# the asyncio run / sleep / gather hasattr + asyncio.run
# return-value layer, and the contextvars module hasattr
# surface layer.
#
# Surface in this fixture:
#   • subprocess — run(cmd, capture_output=True, text=True)
#     returncode / stdout + check_output text-mode +
#     run / Popen / PIPE / STDOUT / DEVNULL /
#     CalledProcessError / TimeoutExpired / check_output /
#     check_call hasattr;
#   • signal — SIGINT / SIGTERM / SIGKILL / SIGUSR1 / SIGUSR2
#     / SIGHUP / SIGPIPE / SIGCHLD / SIGALRM integer-value +
#     SIG_DFL / SIG_IGN / signal / getsignal hasattr;
#   • socket — AF_INET / AF_INET6 / AF_UNIX / SOCK_STREAM /
#     SOCK_DGRAM integer-value + gethostname / gethostbyname
#     hasattr + gethostname() str-return;
#   • selectors — DefaultSelector / EVENT_READ / EVENT_WRITE
#     / BaseSelector / PollSelector hasattr;
#   • asyncio — run / sleep / gather hasattr + asyncio.run
#     on a simple coro returns the int contract;
#   • contextvars — ContextVar / Context / copy_context
#     hasattr.
#
# Behavioral edges that DIVERGE on mamba (subprocess.run
# `check=True` silently passes on a non-zero exit instead of
# raising CalledProcessError, subprocess.Popen(...).communicate
# AttributeError 'Popen' object has no attribute 'communicate',
# socket.socket(AF_INET, SOCK_STREAM) returns a `dict` not a
# socket instance — close / connect surface broken, asyncio.Task
# / Future / Queue / Event / Lock / get_event_loop /
# new_event_loop / set_event_loop hasattr False on mamba —
# entire coroutine-primitive class identifier layer missing,
# contextvars.ContextVar("name", default=...).get raises
# AttributeError 'str' object has no attribute 'get' —
# ContextVar return type is broken) are covered in the
# matching spec fixture
# `lang_subprocess_socket_asyncio_silent`.
import subprocess
import signal
import socket
import selectors
import asyncio
import contextvars


async def _hi():
    return 42


_ledger: list[int] = []

# 1) subprocess — run + capture_output + text
_r = subprocess.run(["echo", "hello"], capture_output=True, text=True)
assert _r.returncode == 0; _ledger.append(1)
assert _r.stdout == "hello\n"; _ledger.append(1)

# 2) subprocess — check_output text-mode
assert subprocess.check_output(["echo", "world"], text=True) == "world\n"; _ledger.append(1)

# 3) subprocess — module attribute hasattr surface
assert hasattr(subprocess, "run") == True; _ledger.append(1)
assert hasattr(subprocess, "Popen") == True; _ledger.append(1)
assert hasattr(subprocess, "PIPE") == True; _ledger.append(1)
assert hasattr(subprocess, "STDOUT") == True; _ledger.append(1)
assert hasattr(subprocess, "DEVNULL") == True; _ledger.append(1)
assert hasattr(subprocess, "CalledProcessError") == True; _ledger.append(1)
assert hasattr(subprocess, "TimeoutExpired") == True; _ledger.append(1)
assert hasattr(subprocess, "check_output") == True; _ledger.append(1)
assert hasattr(subprocess, "check_call") == True; _ledger.append(1)

# 4) signal — integer-value constants
assert signal.SIGINT == 2; _ledger.append(1)
assert signal.SIGTERM == 15; _ledger.append(1)
assert signal.SIGUSR1 == 30; _ledger.append(1)
assert signal.SIGUSR2 == 31; _ledger.append(1)
assert signal.SIGHUP == 1; _ledger.append(1)
assert signal.SIGPIPE == 13; _ledger.append(1)
assert signal.SIGCHLD == 20; _ledger.append(1)
assert signal.SIGALRM == 14; _ledger.append(1)

# 5) signal — module attribute hasattr surface
assert hasattr(signal, "SIGKILL") == True; _ledger.append(1)
assert hasattr(signal, "SIG_DFL") == True; _ledger.append(1)
assert hasattr(signal, "SIG_IGN") == True; _ledger.append(1)
assert hasattr(signal, "signal") == True; _ledger.append(1)
assert hasattr(signal, "getsignal") == True; _ledger.append(1)

# 6) socket — integer-value constants
assert socket.AF_INET == 2; _ledger.append(1)
assert socket.SOCK_STREAM == 1; _ledger.append(1)
assert socket.SOCK_DGRAM == 2; _ledger.append(1)

# 7) socket — module attribute hasattr surface
assert hasattr(socket, "socket") == True; _ledger.append(1)
assert hasattr(socket, "AF_INET") == True; _ledger.append(1)
assert hasattr(socket, "AF_INET6") == True; _ledger.append(1)
assert hasattr(socket, "AF_UNIX") == True; _ledger.append(1)
assert hasattr(socket, "gethostname") == True; _ledger.append(1)
assert hasattr(socket, "gethostbyname") == True; _ledger.append(1)

# 8) socket — gethostname str-return
assert isinstance(socket.gethostname(), str); _ledger.append(1)

# 9) selectors — module attribute hasattr surface
assert hasattr(selectors, "DefaultSelector") == True; _ledger.append(1)
assert hasattr(selectors, "EVENT_READ") == True; _ledger.append(1)
assert hasattr(selectors, "EVENT_WRITE") == True; _ledger.append(1)
assert hasattr(selectors, "BaseSelector") == True; _ledger.append(1)
assert hasattr(selectors, "PollSelector") == True; _ledger.append(1)

# 10) asyncio — module attribute hasattr surface
assert hasattr(asyncio, "run") == True; _ledger.append(1)
assert hasattr(asyncio, "sleep") == True; _ledger.append(1)
assert hasattr(asyncio, "gather") == True; _ledger.append(1)

# 11) asyncio — run on a simple coro
assert asyncio.run(_hi()) == 42; _ledger.append(1)

# 12) contextvars — module attribute hasattr surface
assert hasattr(contextvars, "ContextVar") == True; _ledger.append(1)
assert hasattr(contextvars, "Context") == True; _ledger.append(1)
assert hasattr(contextvars, "copy_context") == True; _ledger.append(1)

# NB: subprocess.run check=True silently passes on a non-zero
# exit, subprocess.Popen(...).communicate AttributeError,
# socket.socket(AF_INET, SOCK_STREAM) returns dict not a
# socket instance, asyncio.Task / Future / Queue / Event /
# Lock / get_event_loop / new_event_loop / set_event_loop
# hasattr False, contextvars.ContextVar("name", default=...).get
# AttributeError 'str' object — all DIVERGE on mamba —
# moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_subprocess_signal_socket_asyncio_value_ops {sum(_ledger)} asserts")
