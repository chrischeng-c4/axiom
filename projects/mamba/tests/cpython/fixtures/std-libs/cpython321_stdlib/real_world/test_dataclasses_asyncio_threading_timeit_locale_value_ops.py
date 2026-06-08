# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_dataclasses_asyncio_threading_timeit_locale_value_ops"
# subject = "cpython321.test_dataclasses_asyncio_threading_timeit_locale_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_dataclasses_asyncio_threading_timeit_locale_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_dataclasses_asyncio_threading_timeit_locale_value_ops: execute CPython 3.12 seed test_dataclasses_asyncio_threading_timeit_locale_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `dataclasses` / `asyncio` / `threading` / `timeit` / `locale`
# five-pack pinned to atomic 204: `dataclasses` (the documented
# partial module-level decorator / helper identifier hasattr
# surface — `dataclass` / `field` / `fields` / `asdict` /
# `astuple`), `asyncio` (the documented partial module-level
# helper / coroutine identifier hasattr surface — `run` /
# `sleep` / `gather` / `wait` / `wait_for` / `shield` /
# `ensure_future` / `create_task`), `threading` (the
# documented full module-level class / helper / sentinel
# identifier hasattr surface — `Thread` / `Lock` / `RLock` /
# `Condition` / `Event` / `Semaphore` / `BoundedSemaphore` /
# `Barrier` / `Timer` / `ThreadError` / `local` /
# `current_thread` / `main_thread` / `active_count` /
# `enumerate` / `get_ident` / `get_native_id` / `settrace` /
# `setprofile` / `stack_size` / `TIMEOUT_MAX` / `excepthook` /
# `ExceptHookArgs` + the documented
# `type(threading.RLock()).__name__ == "RLock"` /
# `type(threading.Event()).__name__ == "Event"` instance
# class identity contract), `timeit` (the documented
# partial module-level helper / class / sentinel
# identifier hasattr surface — `timeit` / `repeat` /
# `default_timer` / `Timer` / `default_number`), and
# `locale` (the documented partial module-level helper /
# constant identifier hasattr surface — `getlocale` /
# `setlocale` / `LC_ALL` / `LC_CTYPE` / `LC_NUMERIC` /
# `LC_TIME` / `format_string` + the documented
# `type(locale.LC_ALL).__name__ == "int"` /
# `isinstance(locale.LC_ALL, int)` constant-type
# contract).
#
# Behavioral edges that DIVERGE on mamba
# (hasattr(dataclasses, "replace") / "Field" /
# "FrozenInstanceError" / "InitVar" / "KW_ONLY" /
# "MISSING" / "make_dataclass" / "is_dataclass" all False
# on mamba, type(threading.Lock()).__name__ collapses to
# "Lock" on mamba instead of "lock", hasattr(timeit,
# "default_repeat") False on mamba,
# type(timeit.default_timer()).__name__ collapses to
# "int" on mamba instead of "float", hasattr(locale,
# "getdefaultlocale") / "getpreferredencoding" /
# "LC_COLLATE" / "LC_MONETARY" / "LC_MESSAGES" /
# "Error" / "localeconv" / "atoi" / "atof" / "currency"
# / "str" / "delocalize" / "normalize" all False on
# mamba) are covered in the matching spec fixture
# `lang_dataclasses_threading_timeit_locale_silent`.
import dataclasses
import asyncio
import threading
import timeit
import locale


_ledger: list[int] = []

# 1) dataclasses — partial module hasattr surface
#    (replace / Field / FrozenInstanceError / InitVar /
#    KW_ONLY / MISSING / make_dataclass / is_dataclass all
#    DIVERGE on mamba — moved to spec)
assert hasattr(dataclasses, "dataclass") == True; _ledger.append(1)
assert hasattr(dataclasses, "field") == True; _ledger.append(1)
assert hasattr(dataclasses, "fields") == True; _ledger.append(1)
assert hasattr(dataclasses, "asdict") == True; _ledger.append(1)
assert hasattr(dataclasses, "astuple") == True; _ledger.append(1)

# 2) asyncio — partial module hasattr surface
assert hasattr(asyncio, "run") == True; _ledger.append(1)
assert hasattr(asyncio, "sleep") == True; _ledger.append(1)
assert hasattr(asyncio, "gather") == True; _ledger.append(1)
assert hasattr(asyncio, "wait") == True; _ledger.append(1)
assert hasattr(asyncio, "wait_for") == True; _ledger.append(1)
assert hasattr(asyncio, "shield") == True; _ledger.append(1)
assert hasattr(asyncio, "ensure_future") == True; _ledger.append(1)
assert hasattr(asyncio, "create_task") == True; _ledger.append(1)

# 3) threading — full module hasattr surface
assert hasattr(threading, "Thread") == True; _ledger.append(1)
assert hasattr(threading, "Lock") == True; _ledger.append(1)
assert hasattr(threading, "RLock") == True; _ledger.append(1)
assert hasattr(threading, "Condition") == True; _ledger.append(1)
assert hasattr(threading, "Event") == True; _ledger.append(1)
assert hasattr(threading, "Semaphore") == True; _ledger.append(1)
assert hasattr(threading, "BoundedSemaphore") == True; _ledger.append(1)
assert hasattr(threading, "Barrier") == True; _ledger.append(1)
assert hasattr(threading, "Timer") == True; _ledger.append(1)
assert hasattr(threading, "ThreadError") == True; _ledger.append(1)
assert hasattr(threading, "local") == True; _ledger.append(1)
assert hasattr(threading, "current_thread") == True; _ledger.append(1)
assert hasattr(threading, "main_thread") == True; _ledger.append(1)
assert hasattr(threading, "active_count") == True; _ledger.append(1)
assert hasattr(threading, "enumerate") == True; _ledger.append(1)
assert hasattr(threading, "get_ident") == True; _ledger.append(1)
assert hasattr(threading, "get_native_id") == True; _ledger.append(1)
assert hasattr(threading, "settrace") == True; _ledger.append(1)
assert hasattr(threading, "setprofile") == True; _ledger.append(1)
assert hasattr(threading, "stack_size") == True; _ledger.append(1)
assert hasattr(threading, "TIMEOUT_MAX") == True; _ledger.append(1)
assert hasattr(threading, "excepthook") == True; _ledger.append(1)
assert hasattr(threading, "ExceptHookArgs") == True; _ledger.append(1)

# 4) threading — instance class identity contract
#    (type(threading.Lock()).__name__ DIVERGES on mamba —
#    moved to spec)
assert type(threading.RLock()).__name__ == "RLock"; _ledger.append(1)
assert type(threading.Event()).__name__ == "Event"; _ledger.append(1)

# 5) timeit — partial module hasattr surface
#    (default_repeat DIVERGES on mamba — moved to spec)
assert hasattr(timeit, "timeit") == True; _ledger.append(1)
assert hasattr(timeit, "repeat") == True; _ledger.append(1)
assert hasattr(timeit, "default_timer") == True; _ledger.append(1)
assert hasattr(timeit, "Timer") == True; _ledger.append(1)
assert hasattr(timeit, "default_number") == True; _ledger.append(1)

# 6) locale — partial module hasattr surface
#    (getdefaultlocale / getpreferredencoding / LC_COLLATE /
#    LC_MONETARY / LC_MESSAGES / Error / localeconv / atoi
#    / atof / currency / str / delocalize / normalize all
#    DIVERGE on mamba — moved to spec)
assert hasattr(locale, "getlocale") == True; _ledger.append(1)
assert hasattr(locale, "setlocale") == True; _ledger.append(1)
assert hasattr(locale, "LC_ALL") == True; _ledger.append(1)
assert hasattr(locale, "LC_CTYPE") == True; _ledger.append(1)
assert hasattr(locale, "LC_NUMERIC") == True; _ledger.append(1)
assert hasattr(locale, "LC_TIME") == True; _ledger.append(1)
assert hasattr(locale, "format_string") == True; _ledger.append(1)

# 7) locale — constant-type contract
assert type(locale.LC_ALL).__name__ == "int"; _ledger.append(1)
assert isinstance(locale.LC_ALL, int) == True; _ledger.append(1)

# NB: hasattr(dataclasses, "replace") / "Field" /
# "FrozenInstanceError" / "InitVar" / "KW_ONLY" /
# "MISSING" / "make_dataclass" / "is_dataclass" all False
# on mamba, type(threading.Lock()).__name__ collapses to
# "Lock" on mamba instead of "lock", hasattr(timeit,
# "default_repeat") False on mamba,
# type(timeit.default_timer()).__name__ collapses to
# "int" on mamba instead of "float", hasattr(locale,
# "getdefaultlocale") / "getpreferredencoding" /
# "LC_COLLATE" / "LC_MONETARY" / "LC_MESSAGES" /
# "Error" / "localeconv" / "atoi" / "atof" / "currency"
# / "str" / "delocalize" / "normalize" all False on
# mamba — all DIVERGE on mamba — moved to the
# divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_dataclasses_asyncio_threading_timeit_locale_value_ops {sum(_ledger)} asserts")
