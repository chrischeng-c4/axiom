# Operational AssertionPass seed for the value contract of five
# bootstrap stdlib modules used by every concurrency / process-
# launch path: `threading` (the documented thread-introspection
# surface — current_thread / active_count / main_thread.name /
# get_ident), `multiprocessing` (the documented cpu_count
# helper + Manager / Pipe attribute surface),
# `asyncio` (the documented top-level helper surface — run /
# gather / sleep / wait / create_task + the coroutine event-
# loop run contract), `concurrent.futures` (the
# `as_completed` documented helper), and `subprocess` (the
# documented PIPE / STDOUT / DEVNULL constants + run / Popen
# / CalledProcessError / TimeoutExpired hasattr surface + the
# `run(["echo", "hello"])` capture_output / text round-trip).
#
# The matching subset between mamba and CPython is the
# thread-introspection layer + cpu-count layer + asyncio top-
# level surface + concurrent.futures helper attribute layer +
# subprocess execve helper layer: threading.current_thread()
# returns an object; threading.active_count() returns 1 in
# the single-thread baseline; threading.main_thread().name ==
# "MainThread"; threading.get_ident() returns an int;
# multiprocessing.cpu_count() returns an int; asyncio.run on
# a coroutine `return 42` returns 42; subprocess.run(["echo",
# "hello"], capture_output=True, text=True).stdout ==
# "hello\n" + returncode == 0; subprocess.PIPE / STDOUT /
# DEVNULL == -1 / -2 / -3.
#
# Surface in this fixture:
#   • threading.current_thread() returns a non-None object;
#   • threading.active_count() == 1 (single-thread baseline);
#   • threading.main_thread().name == "MainThread";
#   • type(threading.get_ident()).__name__ == "int";
#   • type(multiprocessing.cpu_count()).__name__ == "int";
#   • asyncio — hasattr run / gather / sleep / wait /
#     create_task;
#   • asyncio.run on a coroutine returns the coroutine value;
#   • concurrent.futures — hasattr as_completed;
#   • subprocess.PIPE / STDOUT / DEVNULL == -1 / -2 / -3;
#   • subprocess — hasattr run / Popen / CalledProcessError /
#     TimeoutExpired;
#   • subprocess.run(["echo", "hello"], capture_output=True,
#     text=True) — returncode == 0 + stdout == "hello\n".
#
# Behavioral edges that DIVERGE on mamba (threading.Thread /
# Event / Condition / Semaphore / BoundedSemaphore / Timer /
# Barrier / local class identity, threading.Lock().acquire /
# release AttributeError, multiprocessing.Process / Pool /
# Queue class identity, multiprocessing.Lock construction
# broken, asyncio.Future / Task / Event / Queue / Lock /
# Semaphore / TimeoutError / CancelledError hasattr False,
# concurrent.futures.ThreadPoolExecutor / ProcessPoolExecutor
# / Future class identity, concurrent.futures.wait /
# ALL_COMPLETED / FIRST_COMPLETED / FIRST_EXCEPTION attribute
# surface, subprocess.Popen / CompletedProcess /
# CalledProcessError class identity) are covered in the
# matching spec fixture
# `lang_threading_asyncio_futures_silent`.
import threading
import multiprocessing
import asyncio
import concurrent.futures as cf
import subprocess


_ledger: list[int] = []

# 1) threading — current_thread / main_thread introspection
assert threading.current_thread() is not None; _ledger.append(1)
assert threading.active_count() == 1; _ledger.append(1)
assert threading.main_thread().name == "MainThread"; _ledger.append(1)

# 2) threading — get_ident returns int
assert type(threading.get_ident()).__name__ == "int"; _ledger.append(1)

# 3) multiprocessing — cpu_count int
assert type(multiprocessing.cpu_count()).__name__ == "int"; _ledger.append(1)
assert multiprocessing.cpu_count() >= 1; _ledger.append(1)

# 4) asyncio hasattr — top-level helper surface
assert hasattr(asyncio, "run"); _ledger.append(1)
assert hasattr(asyncio, "gather"); _ledger.append(1)
assert hasattr(asyncio, "sleep"); _ledger.append(1)
assert hasattr(asyncio, "wait"); _ledger.append(1)
assert hasattr(asyncio, "create_task"); _ledger.append(1)

# 5) asyncio.run — coroutine event-loop integration
async def _coro():
    return 42

assert asyncio.run(_coro()) == 42; _ledger.append(1)

# 6) concurrent.futures — as_completed helper surface
assert hasattr(cf, "as_completed"); _ledger.append(1)

# 7) subprocess — PIPE / STDOUT / DEVNULL sentinels
assert subprocess.PIPE == -1; _ledger.append(1)
assert subprocess.STDOUT == -2; _ledger.append(1)
assert subprocess.DEVNULL == -3; _ledger.append(1)

# 8) subprocess hasattr — execve helper surface
assert hasattr(subprocess, "run"); _ledger.append(1)
assert hasattr(subprocess, "Popen"); _ledger.append(1)
assert hasattr(subprocess, "CalledProcessError"); _ledger.append(1)
assert hasattr(subprocess, "TimeoutExpired"); _ledger.append(1)

# 9) subprocess.run — execve + capture_output round-trip
_result = subprocess.run(["echo", "hello"], capture_output=True, text=True)
assert _result.returncode == 0; _ledger.append(1)
assert _result.stdout == "hello\n"; _ledger.append(1)

# NB: threading.Thread / Event / Condition / Semaphore /
# BoundedSemaphore / Timer / Barrier / local class identity,
# threading.Lock().acquire / release AttributeError,
# multiprocessing.Process / Pool / Queue class identity,
# multiprocessing.Lock construction broken, asyncio.Future /
# Task / Event / Queue / Lock / Semaphore / TimeoutError /
# CancelledError hasattr False, concurrent.futures.
# ThreadPoolExecutor / ProcessPoolExecutor / Future class
# identity, concurrent.futures.wait / ALL_COMPLETED /
# FIRST_COMPLETED / FIRST_EXCEPTION attribute surface,
# subprocess.Popen / CompletedProcess / CalledProcessError
# class identity all DIVERGE on mamba — moved to the
# divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_concurrency_subprocess_value_ops {sum(_ledger)} asserts")
