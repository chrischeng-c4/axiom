# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_subprocess_socket_asyncio_silent"
# subject = "cpython321.lang_subprocess_socket_asyncio_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_subprocess_socket_asyncio_silent.py"
# status = "filled"
# ///
"""cpython321.lang_subprocess_socket_asyncio_silent: execute CPython 3.12 seed lang_subprocess_socket_asyncio_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across the
# subprocess-spawn / socket-create / coroutine-primitive /
# context-variable quad pinned by atomic 171: `subprocess` (the
# documented `run(cmd, check=True)` raise-on-non-zero-exit
# contract + the documented `Popen(...).communicate` instance
# method), `socket` (the documented `socket(AF_INET,
# SOCK_STREAM)` instance constructor that returns a `socket.
# socket` instance), `asyncio` (the documented `Task` /
# `Future` / `Queue` / `Event` / `Lock` / `get_event_loop` /
# `new_event_loop` / `set_event_loop` module-level identifier
# surface), and `contextvars` (the documented `ContextVar(name,
# default=...).get` instance method).
#
# The matching subset (subprocess run + capture_output + text +
# returncode / stdout + check_output + module hasattr surface,
# signal int-constant + module hasattr surface, socket int-
# constant + module hasattr surface + gethostname str-return,
# selectors module hasattr surface, asyncio run / sleep /
# gather hasattr + run-coro return-value, contextvars module
# hasattr surface) is covered by
# `test_subprocess_signal_socket_asyncio_value_ops`; this
# fixture pins the CPython-only contracts that mamba currently
# elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • subprocess.run(["false"], check=True) raises
#     CalledProcessError on a non-zero exit — documented
#     check= keyword contract (mamba: returns silently — the
#     non-zero exit is not surfaced as an exception);
#   • subprocess.Popen(["cat"], stdin=PIPE, stdout=PIPE,
#     text=True).communicate("hi\n") returns ("hi\n", None) —
#     documented Popen instance method (mamba: AttributeError,
#     'Popen' object has no attribute 'communicate');
#   • type(socket.socket(AF_INET, SOCK_STREAM)).__name__ ==
#     "socket" — documented socket-instance constructor
#     contract (mamba: returns a `dict` — the entire socket
#     instance surface is broken);
#   • hasattr(asyncio, "Task") / "Future" / "Queue" / "Event" /
#     "Lock" / "get_event_loop" / "new_event_loop" /
#     "set_event_loop" is True — documented module-level
#     class / event-loop helper identifier surface (mamba:
#     False — the entire coroutine-primitive class identifier
#     layer is missing);
#   • contextvars.ContextVar("name", default=10).get() == 10
#     — documented ContextVar instance method (mamba:
#     AttributeError 'str' object has no attribute 'get' —
#     ContextVar() return type is broken).
import subprocess as _subprocess_mod
import socket as _socket_mod
import asyncio as _asyncio_mod
import contextvars as _contextvars_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / instance methods / event-loop helpers
# that mamba's bundled type stubs do not surface accurately.
subprocess: Any = _subprocess_mod
socket: Any = _socket_mod
asyncio: Any = _asyncio_mod
contextvars: Any = _contextvars_mod


_ledger: list[int] = []

# 1) subprocess.run — check=True raises CalledProcessError
_raised = False
try:
    subprocess.run(["false"], check=True)
except subprocess.CalledProcessError:
    _raised = True
assert _raised == True; _ledger.append(1)

# 2) subprocess.Popen — communicate instance method
_p = subprocess.Popen(["cat"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
_stdout, _stderr = _p.communicate("hi\n")
assert _stdout == "hi\n"; _ledger.append(1)
assert _stderr == None; _ledger.append(1)
assert _p.returncode == 0; _ledger.append(1)

# 3) socket.socket — instance constructor
_s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
assert type(_s).__name__ == "socket"; _ledger.append(1)
_s.close()

# 4) asyncio — module-level class / event-loop helper surface
assert hasattr(asyncio, "Task") == True; _ledger.append(1)
assert hasattr(asyncio, "Future") == True; _ledger.append(1)
assert hasattr(asyncio, "Queue") == True; _ledger.append(1)
assert hasattr(asyncio, "Event") == True; _ledger.append(1)
assert hasattr(asyncio, "Lock") == True; _ledger.append(1)
assert hasattr(asyncio, "get_event_loop") == True; _ledger.append(1)
assert hasattr(asyncio, "new_event_loop") == True; _ledger.append(1)
assert hasattr(asyncio, "set_event_loop") == True; _ledger.append(1)

# 5) contextvars — ContextVar instance .get method
_cv = contextvars.ContextVar("test", default=10)
assert _cv.get() == 10; _ledger.append(1)
_cv.set(20)
assert _cv.get() == 20; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_subprocess_socket_asyncio_silent {sum(_ledger)} asserts")
