# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_threading_asyncio_concurrent_silent"
# subject = "cpython321.lang_threading_asyncio_concurrent_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_threading_asyncio_concurrent_silent.py"
# status = "filled"
# ///
"""cpython321.lang_threading_asyncio_concurrent_silent: execute CPython 3.12 seed lang_threading_asyncio_concurrent_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the `threading` /
# `asyncio` / `concurrent.futures` three-pack pinned to atomic
# 216: `threading` (the documented
# `type(threading.current_thread()).__name__ == "_MainThread"`
# main-thread type-identity value contract + the documented
# `type(threading.main_thread()).__name__ == "_MainThread"`
# main-thread type-identity value contract + the documented
# `type(threading.Lock()).__name__ == "lock"` Lock-instance
# type-identity value contract + the documented
# `type(threading.local()).__name__ == "_local"` thread-local
# type-identity value contract + the documented
# `type(threading.Thread(target=...)).__name__ == "Thread"`
# Thread-instance type-identity value contract),
# `asyncio` (the documented
# `hasattr(asyncio, "get_event_loop") / "new_event_loop" /
# "set_event_loop" / "Task" / "Future" / "Queue" / "Lock" /
# "Event" / "Semaphore" / "TimeoutError" / "CancelledError" /
# "iscoroutine" / "iscoroutinefunction" / "all_tasks" /
# "current_task" == True` extended hasattr surface), and
# `concurrent.futures` (the documented
# `hasattr(concurrent.futures, "Future") / "Executor" /
# "ThreadPoolExecutor" / "ProcessPoolExecutor" /
# "as_completed" / "wait" / "FIRST_COMPLETED" /
# "FIRST_EXCEPTION" / "ALL_COMPLETED" / "CancelledError" /
# "TimeoutError" / "BrokenExecutor" == True` full module-
# level helper / class / sentinel identifier hasattr
# surface).
#
# Behavioral edges that CONFORM on mamba
# (threading `Thread` / `Lock` / `RLock` / `Condition` /
# `Semaphore` / `BoundedSemaphore` / `Event` / `Timer` /
# `Barrier` / `local` / `current_thread` / `main_thread` /
# `active_count` / `enumerate` / `get_ident` /
# `get_native_id` / `stack_size` / `setprofile` / `settrace`
# / `excepthook` / `ExceptHookArgs` full hasattr surface +
# `threading.active_count() >= 1` /
# `type(threading.active_count()).__name__ == "int"` /
# `threading.get_ident() > 0` /
# `type(threading.get_ident()).__name__ == "int"` /
# `type(threading.Event()).__name__ == "Event"`
# introspection value contract, asyncio partial hasattr
# surface `run` / `gather` / `sleep` / `wait` / `wait_for`
# / `create_task` / `ensure_future` +
# `asyncio.run(coro_returning_7) == 7` event-loop value
# contract, graphlib full hasattr surface +
# `hasattr(graphlib, "TopologicalSorter") == True` /
# `hasattr(graphlib, "CycleError") == True`, email
# `message_from_string` / `message_from_bytes` hasattr
# surface) are covered in the matching pass fixture
# `test_threading_asyncio_graphlib_email_value_ops`.
from typing import Any
import threading as _threading_mod
import asyncio as _asyncio_mod
import concurrent.futures as _cf_mod

threading: Any = _threading_mod
asyncio: Any = _asyncio_mod
cf: Any = _cf_mod


_ledger: list[int] = []

# 1) threading — main-thread type-identity value contract
#    (mamba: type(threading.current_thread()).__name__
#    "_MainThread" collapses to "Thread")
assert type(threading.current_thread()).__name__ == "_MainThread"; _ledger.append(1)

# 2) threading — main-thread type-identity value contract
#    (mamba: type(threading.main_thread()).__name__
#    "_MainThread" collapses to "Thread")
assert type(threading.main_thread()).__name__ == "_MainThread"; _ledger.append(1)

# 3) threading — Lock-instance type-identity value contract
#    (mamba: type(threading.Lock()).__name__ "lock"
#    collapses to "Lock" — CPython uses lowercase, mamba uses
#    capital)
assert type(threading.Lock()).__name__ == "lock"; _ledger.append(1)

# 4) threading — thread-local type-identity value contract
#    (mamba: type(threading.local()).__name__ "_local"
#    collapses to "dict")
assert type(threading.local()).__name__ == "_local"; _ledger.append(1)

# 5) threading — Thread-instance type-identity value contract
#    (mamba: type(threading.Thread(target=...)).__name__
#    "Thread" collapses to "dict" — even Thread itself
#    instantiates to a dict on mamba)
def _noop() -> None:
    return None
_th = threading.Thread(target=_noop)
assert type(_th).__name__ == "Thread"; _ledger.append(1)

# 6) asyncio — extended module hasattr surface
#    (mamba: get_event_loop / new_event_loop / set_event_loop
#    / Task / Future / Queue / Lock / Event / Semaphore /
#    TimeoutError / CancelledError / iscoroutine /
#    iscoroutinefunction / all_tasks / current_task all
#    False)
assert hasattr(asyncio, "get_event_loop") == True; _ledger.append(1)
assert hasattr(asyncio, "new_event_loop") == True; _ledger.append(1)
assert hasattr(asyncio, "set_event_loop") == True; _ledger.append(1)
assert hasattr(asyncio, "Task") == True; _ledger.append(1)
assert hasattr(asyncio, "Future") == True; _ledger.append(1)
assert hasattr(asyncio, "Queue") == True; _ledger.append(1)
assert hasattr(asyncio, "Lock") == True; _ledger.append(1)
assert hasattr(asyncio, "Event") == True; _ledger.append(1)
assert hasattr(asyncio, "Semaphore") == True; _ledger.append(1)
assert hasattr(asyncio, "TimeoutError") == True; _ledger.append(1)
assert hasattr(asyncio, "CancelledError") == True; _ledger.append(1)
assert hasattr(asyncio, "iscoroutine") == True; _ledger.append(1)
assert hasattr(asyncio, "iscoroutinefunction") == True; _ledger.append(1)
assert hasattr(asyncio, "all_tasks") == True; _ledger.append(1)
assert hasattr(asyncio, "current_task") == True; _ledger.append(1)

# 7) concurrent.futures — full module-level helper / class /
#    sentinel identifier hasattr surface
#    (mamba: Future / Executor / ThreadPoolExecutor /
#    ProcessPoolExecutor / as_completed / wait /
#    FIRST_COMPLETED / FIRST_EXCEPTION / ALL_COMPLETED /
#    CancelledError / TimeoutError / BrokenExecutor all
#    False)
assert hasattr(cf, "Future") == True; _ledger.append(1)
assert hasattr(cf, "Executor") == True; _ledger.append(1)
assert hasattr(cf, "ThreadPoolExecutor") == True; _ledger.append(1)
assert hasattr(cf, "ProcessPoolExecutor") == True; _ledger.append(1)
assert hasattr(cf, "as_completed") == True; _ledger.append(1)
assert hasattr(cf, "wait") == True; _ledger.append(1)
assert hasattr(cf, "FIRST_COMPLETED") == True; _ledger.append(1)
assert hasattr(cf, "FIRST_EXCEPTION") == True; _ledger.append(1)
assert hasattr(cf, "ALL_COMPLETED") == True; _ledger.append(1)
assert hasattr(cf, "CancelledError") == True; _ledger.append(1)
assert hasattr(cf, "TimeoutError") == True; _ledger.append(1)
assert hasattr(cf, "BrokenExecutor") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_threading_asyncio_concurrent_silent {sum(_ledger)} asserts")
