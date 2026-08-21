# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_threading_asyncio_graphlib_email_value_ops"
# subject = "cpython321.test_threading_asyncio_graphlib_email_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_threading_asyncio_graphlib_email_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_threading_asyncio_graphlib_email_value_ops: execute CPython 3.12 seed test_threading_asyncio_graphlib_email_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `threading` / `asyncio` / `graphlib` / `email` four-pack pinned
# to atomic 216: `threading` (the documented full module-level
# helper / class / sentinel identifier hasattr surface — `Thread`
# / `Lock` / `RLock` / `Condition` / `Semaphore` /
# `BoundedSemaphore` / `Event` / `Timer` / `Barrier` / `local` /
# `current_thread` / `main_thread` / `active_count` / `enumerate`
# / `get_ident` / `get_native_id` / `stack_size` / `setprofile`
# / `settrace` / `excepthook` / `ExceptHookArgs` + the documented
# `threading.active_count() >= 1` /
# `type(threading.active_count()).__name__ == "int"` /
# `threading.get_ident() > 0` /
# `type(threading.get_ident()).__name__ == "int"` /
# `type(threading.Event()).__name__ == "Event"` runtime-
# introspection value contract), `asyncio` (the documented
# partial module-level helper identifier hasattr surface —
# `run` / `gather` / `sleep` / `wait` / `wait_for` /
# `create_task` / `ensure_future` + the documented
# `asyncio.run(coro_returning_7) == 7` event-loop value
# contract), `graphlib` (the documented full module-level
# class identifier hasattr surface — `TopologicalSorter` /
# `CycleError`), and `email` (the documented partial module-
# level helper identifier hasattr surface —
# `message_from_string` / `message_from_bytes`).
#
# Behavioral edges that DIVERGE on mamba
# (type(threading.current_thread()).__name__ == "_MainThread"
# collapses to "Thread" on mamba +
# type(threading.main_thread()).__name__ == "_MainThread"
# collapses to "Thread" on mamba +
# type(threading.Lock()).__name__ == "lock" collapses to
# "Lock" on mamba + type(threading.local()).__name__ ==
# "_local" collapses to "dict" on mamba +
# type(threading.Thread(target=...)).__name__ == "Thread"
# collapses to "dict" on mamba, hasattr(asyncio,
# "get_event_loop") / "new_event_loop" / "set_event_loop" /
# "Task" / "Future" / "Queue" / "Lock" / "Event" / "Semaphore"
# / "TimeoutError" / "CancelledError" / "iscoroutine" /
# "iscoroutinefunction" / "all_tasks" / "current_task" all
# False on mamba, hasattr(concurrent.futures, "Future") /
# "Executor" / "ThreadPoolExecutor" / "ProcessPoolExecutor" /
# "as_completed" / "wait" / "FIRST_COMPLETED" /
# "FIRST_EXCEPTION" / "ALL_COMPLETED" / "CancelledError" /
# "TimeoutError" / "BrokenExecutor" all False on mamba) are
# covered in the matching spec fixture
# `lang_threading_asyncio_concurrent_silent`.
import threading
import asyncio
import graphlib
import email


_ledger: list[int] = []

# 1) threading — full module hasattr surface
assert hasattr(threading, "Thread") == True; _ledger.append(1)
assert hasattr(threading, "Lock") == True; _ledger.append(1)
assert hasattr(threading, "RLock") == True; _ledger.append(1)
assert hasattr(threading, "Condition") == True; _ledger.append(1)
assert hasattr(threading, "Semaphore") == True; _ledger.append(1)
assert hasattr(threading, "BoundedSemaphore") == True; _ledger.append(1)
assert hasattr(threading, "Event") == True; _ledger.append(1)
assert hasattr(threading, "Timer") == True; _ledger.append(1)
assert hasattr(threading, "Barrier") == True; _ledger.append(1)
assert hasattr(threading, "local") == True; _ledger.append(1)
assert hasattr(threading, "current_thread") == True; _ledger.append(1)
assert hasattr(threading, "main_thread") == True; _ledger.append(1)
assert hasattr(threading, "active_count") == True; _ledger.append(1)
assert hasattr(threading, "enumerate") == True; _ledger.append(1)
assert hasattr(threading, "get_ident") == True; _ledger.append(1)
assert hasattr(threading, "get_native_id") == True; _ledger.append(1)
assert hasattr(threading, "stack_size") == True; _ledger.append(1)
assert hasattr(threading, "setprofile") == True; _ledger.append(1)
assert hasattr(threading, "settrace") == True; _ledger.append(1)
assert hasattr(threading, "excepthook") == True; _ledger.append(1)
assert hasattr(threading, "ExceptHookArgs") == True; _ledger.append(1)

# 2) threading — runtime-introspection value contract
#    (type(threading.current_thread()).__name__ "_MainThread"
#    / type(threading.main_thread()).__name__ "_MainThread"
#    / type(threading.Lock()).__name__ "lock" /
#    type(threading.local()).__name__ "_local" /
#    type(threading.Thread(...)).__name__ "Thread" all
#    DIVERGE on mamba — moved to spec)
assert threading.active_count() >= 1; _ledger.append(1)
assert type(threading.active_count()).__name__ == "int"; _ledger.append(1)
assert threading.get_ident() > 0; _ledger.append(1)
assert type(threading.get_ident()).__name__ == "int"; _ledger.append(1)
assert type(threading.Event()).__name__ == "Event"; _ledger.append(1)

# 3) asyncio — partial module hasattr surface
#    (get_event_loop / new_event_loop / set_event_loop /
#    Task / Future / Queue / Lock / Event / Semaphore /
#    TimeoutError / CancelledError / iscoroutine /
#    iscoroutinefunction / all_tasks / current_task all
#    DIVERGE on mamba — moved to spec)
assert hasattr(asyncio, "run") == True; _ledger.append(1)
assert hasattr(asyncio, "gather") == True; _ledger.append(1)
assert hasattr(asyncio, "sleep") == True; _ledger.append(1)
assert hasattr(asyncio, "wait") == True; _ledger.append(1)
assert hasattr(asyncio, "wait_for") == True; _ledger.append(1)
assert hasattr(asyncio, "create_task") == True; _ledger.append(1)
assert hasattr(asyncio, "ensure_future") == True; _ledger.append(1)

# 4) asyncio — event-loop value contract
async def _aco() -> int:
    return 7
assert asyncio.run(_aco()) == 7; _ledger.append(1)

# 5) graphlib — full module hasattr surface
assert hasattr(graphlib, "TopologicalSorter") == True; _ledger.append(1)
assert hasattr(graphlib, "CycleError") == True; _ledger.append(1)

# 6) email — partial module hasattr surface
assert hasattr(email, "message_from_string") == True; _ledger.append(1)
assert hasattr(email, "message_from_bytes") == True; _ledger.append(1)

# NB: type(threading.current_thread()).__name__ == "_MainThread"
# collapses to "Thread" on mamba +
# type(threading.main_thread()).__name__ == "_MainThread"
# collapses to "Thread" on mamba +
# type(threading.Lock()).__name__ == "lock" collapses to
# "Lock" on mamba + type(threading.local()).__name__ ==
# "_local" collapses to "dict" on mamba +
# type(threading.Thread(target=...)).__name__ == "Thread"
# collapses to "dict" on mamba, hasattr(asyncio,
# "get_event_loop") / "new_event_loop" / "set_event_loop"
# / "Task" / "Future" / "Queue" / "Lock" / "Event" /
# "Semaphore" / "TimeoutError" / "CancelledError" /
# "iscoroutine" / "iscoroutinefunction" / "all_tasks" /
# "current_task" all False on mamba,
# hasattr(concurrent.futures, "Future") / "Executor" /
# "ThreadPoolExecutor" / "ProcessPoolExecutor" /
# "as_completed" / "wait" / "FIRST_COMPLETED" /
# "FIRST_EXCEPTION" / "ALL_COMPLETED" / "CancelledError" /
# "TimeoutError" / "BrokenExecutor" all False on mamba —
# all DIVERGE on mamba — moved to the divergence-spec
# fixture.

print(f"MAMBA_ASSERTION_PASS: test_threading_asyncio_graphlib_email_value_ops {sum(_ledger)} asserts")
