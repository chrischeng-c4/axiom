# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_threading_asyncio_futures_silent"
# subject = "cpython321.lang_threading_asyncio_futures_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_threading_asyncio_futures_silent.py"
# status = "filled"
# ///
"""cpython321.lang_threading_asyncio_futures_silent: execute CPython 3.12 seed lang_threading_asyncio_futures_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across the
# concurrency / IPC / future / coroutine / subprocess quintet
# pinned by atomic 154: `threading` (the documented Thread /
# Event / Condition / Semaphore / Barrier / Timer / local
# bare-class identity + the `Lock().acquire / release`
# instance methods), `multiprocessing` (the documented
# Manager / Pipe attribute surface + Process / Pool / Queue
# class identity), `asyncio` (the documented Future / Task /
# Event / Queue / Lock / Semaphore / TimeoutError /
# CancelledError attribute surface — the high-level coroutine
# primitives), `concurrent.futures` (the documented
# ThreadPoolExecutor / ProcessPoolExecutor / Future class
# identity + the documented `wait` helper + the
# ALL_COMPLETED / FIRST_COMPLETED / FIRST_EXCEPTION
# completion-mode sentinels), and `subprocess` (the
# documented Popen / CompletedProcess / CalledProcessError
# bare-class identity).
#
# The matching subset (threading.current_thread /
# active_count / main_thread.name / get_ident,
# multiprocessing.cpu_count returning int + >= 1, asyncio.run
# / gather / sleep / wait / create_task hasattr +
# asyncio.run on a coroutine return value,
# concurrent.futures.as_completed hasattr, subprocess.PIPE /
# STDOUT / DEVNULL sentinels + run / Popen /
# CalledProcessError / TimeoutExpired hasattr +
# subprocess.run echo round-trip) is covered by
# `test_concurrency_subprocess_value_ops`; this fixture pins
# the CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • threading.Thread.__name__ == "Thread" — bare class
#     identity (mamba: returns None);
#   • threading.Event.__name__ == "Event" (mamba: None);
#   • threading.Condition.__name__ == "Condition" (mamba:
#     None);
#   • threading.Semaphore.__name__ == "Semaphore" (mamba:
#     None);
#   • threading.BoundedSemaphore.__name__ ==
#     "BoundedSemaphore" (mamba: None);
#   • threading.Timer.__name__ == "Timer" (mamba: None);
#   • threading.Barrier.__name__ == "Barrier" (mamba: None);
#   • threading.local.__name__ == "_local" — thread-local
#     storage class identity (mamba: None);
#   • threading.Lock() exposes the documented acquire /
#     release methods (mamba: AttributeError, 'Lock' object
#     has no attribute 'acquire');
#   • hasattr(multiprocessing, "Manager") is True (mamba:
#     returns False, AttributeError raised internally);
#   • hasattr(multiprocessing, "Pipe") is True (mamba:
#     False);
#   • multiprocessing.Process.__name__ == "Process" (mamba:
#     None);
#   • multiprocessing.Pool.__name__ == "Pool" (mamba: None);
#   • multiprocessing.Queue.__name__ == "Queue" (mamba:
#     None);
#   • hasattr(asyncio, "Future") is True — high-level
#     coroutine-primitive surface (mamba: False);
#   • hasattr(asyncio, "Task") is True (mamba: False);
#   • hasattr(asyncio, "Event") is True (mamba: False);
#   • hasattr(asyncio, "Queue") is True (mamba: False);
#   • hasattr(asyncio, "Lock") is True (mamba: False);
#   • hasattr(asyncio, "Semaphore") is True (mamba: False);
#   • hasattr(asyncio, "CancelledError") is True (mamba:
#     False);
#   • concurrent.futures.ThreadPoolExecutor.__name__ ==
#     "ThreadPoolExecutor" (mamba: None);
#   • concurrent.futures.ProcessPoolExecutor.__name__ ==
#     "ProcessPoolExecutor" (mamba: None);
#   • concurrent.futures.Future.__name__ == "Future" (mamba:
#     None);
#   • hasattr(concurrent.futures, "wait") is True (mamba:
#     False);
#   • concurrent.futures.ALL_COMPLETED == "ALL_COMPLETED" —
#     completion-mode sentinel (mamba: None);
#   • concurrent.futures.FIRST_COMPLETED == "FIRST_COMPLETED"
#     (mamba: None);
#   • concurrent.futures.FIRST_EXCEPTION == "FIRST_EXCEPTION"
#     (mamba: None);
#   • subprocess.Popen.__name__ == "Popen" (mamba: None);
#   • subprocess.CompletedProcess.__name__ ==
#     "CompletedProcess" (mamba: None);
#   • subprocess.CalledProcessError.__name__ ==
#     "CalledProcessError" (mamba: None).
import threading as _threading_mod
import multiprocessing as _multiprocessing_mod
import asyncio as _asyncio_mod
import concurrent.futures as _cf_mod
import subprocess as _subprocess_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / module-level helpers / instance methods
# that mamba's bundled type stubs do not surface accurately.
threading: Any = _threading_mod
multiprocessing: Any = _multiprocessing_mod
asyncio: Any = _asyncio_mod
cf: Any = _cf_mod
subprocess: Any = _subprocess_mod


_ledger: list[int] = []

# 1) threading — bare class identity surface
assert threading.Thread.__name__ == "Thread"; _ledger.append(1)
assert threading.Event.__name__ == "Event"; _ledger.append(1)
assert threading.Condition.__name__ == "Condition"; _ledger.append(1)
assert threading.Semaphore.__name__ == "Semaphore"; _ledger.append(1)
assert threading.BoundedSemaphore.__name__ == "BoundedSemaphore"; _ledger.append(1)
assert threading.Timer.__name__ == "Timer"; _ledger.append(1)
assert threading.Barrier.__name__ == "Barrier"; _ledger.append(1)
assert threading.local.__name__ == "_local"; _ledger.append(1)

# 2) threading.Lock — instance acquire / release contract
_lk = threading.Lock()
_lk.acquire()
_lk.release()
assert True; _ledger.append(1)

# 3) multiprocessing — Manager / Pipe attribute surface
assert hasattr(multiprocessing, "Manager") == True; _ledger.append(1)
assert hasattr(multiprocessing, "Pipe") == True; _ledger.append(1)

# 4) multiprocessing — bare class identity surface
assert multiprocessing.Process.__name__ == "Process"; _ledger.append(1)
assert multiprocessing.Pool.__name__ == "Pool"; _ledger.append(1)
assert multiprocessing.Queue.__name__ == "Queue"; _ledger.append(1)

# 5) asyncio — high-level coroutine-primitive surface
assert hasattr(asyncio, "Future") == True; _ledger.append(1)
assert hasattr(asyncio, "Task") == True; _ledger.append(1)
assert hasattr(asyncio, "Event") == True; _ledger.append(1)
assert hasattr(asyncio, "Queue") == True; _ledger.append(1)
assert hasattr(asyncio, "Lock") == True; _ledger.append(1)
assert hasattr(asyncio, "Semaphore") == True; _ledger.append(1)
assert hasattr(asyncio, "CancelledError") == True; _ledger.append(1)

# 6) concurrent.futures — executor class identity
assert cf.ThreadPoolExecutor.__name__ == "ThreadPoolExecutor"; _ledger.append(1)
assert cf.ProcessPoolExecutor.__name__ == "ProcessPoolExecutor"; _ledger.append(1)
assert cf.Future.__name__ == "Future"; _ledger.append(1)

# 7) concurrent.futures — wait helper + completion-mode sentinels
assert hasattr(cf, "wait") == True; _ledger.append(1)
assert cf.ALL_COMPLETED == "ALL_COMPLETED"; _ledger.append(1)
assert cf.FIRST_COMPLETED == "FIRST_COMPLETED"; _ledger.append(1)
assert cf.FIRST_EXCEPTION == "FIRST_EXCEPTION"; _ledger.append(1)

# 8) subprocess — execve class identity surface
assert subprocess.Popen.__name__ == "Popen"; _ledger.append(1)
assert subprocess.CompletedProcess.__name__ == "CompletedProcess"; _ledger.append(1)
assert subprocess.CalledProcessError.__name__ == "CalledProcessError"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_threading_asyncio_futures_silent {sum(_ledger)} asserts")
