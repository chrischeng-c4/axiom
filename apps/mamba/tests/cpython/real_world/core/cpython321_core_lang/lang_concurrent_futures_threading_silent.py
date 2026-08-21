# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_concurrent_futures_threading_silent"
# subject = "cpython321.lang_concurrent_futures_threading_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_concurrent_futures_threading_silent.py"
# status = "filled"
# ///
"""cpython321.lang_concurrent_futures_threading_silent: execute CPython 3.12 seed lang_concurrent_futures_threading_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(concurrent.futures, '
# ThreadPoolExecutor')` (the documented "concurrent.futures exposes
# the ThreadPoolExecutor class" — mamba returns False),
# `hasattr(concurrent.futures, 'ProcessPoolExecutor')` (the
# documented "concurrent.futures exposes the ProcessPoolExecutor
# class" — mamba returns False), `hasattr(concurrent.futures, '
# Future')` (the documented "concurrent.futures exposes the Future
# class" — mamba returns False), `hasattr(concurrent.futures, '
# Executor')` (the documented "concurrent.futures exposes the
# Executor base class" — mamba returns False), `hasattr(concurrent.
# futures, 'as_completed')` (the documented "concurrent.futures
# exposes the as_completed iterator" — mamba returns False), `hasattr
# (concurrent.futures, 'wait')` (the documented "concurrent.futures
# exposes the wait helper" — mamba returns False), `hasattr(
# concurrent.futures, 'CancelledError')` (the documented
# "concurrent.futures exposes the CancelledError exception" — mamba
# returns False), `type(threading.Lock()).__name__ == 'lock'` (the
# documented "threading.Lock() returns a lock instance whose type
# name is the lowercase 'lock'" — mamba returns 'Lock' — capitalised
# type-name placeholder), `type(threading.current_thread()).__name__
# == '_MainThread'` (the documented "current_thread() in the main
# thread returns an instance of the internal _MainThread class" —
# mamba returns 'Thread' — bare base-class type name), and `type(
# threading.local()).__name__ == '_local'` (the documented
# "threading.local() returns a thread-local-storage object of type
# _local" — mamba returns 'dict' — constructor degrades to plain
# dict).
# Ten-pack pinned to atomic 305.
#
# Behavioral edges that CONFORM on mamba (threading — hasattr Thread/
# Lock/RLock/Condition/Semaphore/BoundedSemaphore/Event/Timer/Barrier/
# local/current_thread/active_count/enumerate/main_thread/get_ident +
# active_count == 1 in main + get_ident returns int. runpy — hasattr
# run_module/run_path. pkgutil — hasattr iter_modules/walk_packages/
# get_data/ModuleInfo/resolve_name. doctest — hasattr testmod/test
# file/DocTestSuite/DocFileSuite/run_docstring_examples/Example/Doc
# Test/DocTestParser/DocTestRunner/DebugRunner/OutputChecker/ELLIPSIS/
# IGNORE_EXCEPTION_DETAIL/SKIP) are covered in the matching pass
# fixture `test_threading_runpy_doctest_value_ops`.
from concurrent import futures
import threading


_ledger: list[int] = []

# 1) hasattr(concurrent.futures, 'ThreadPoolExecutor') — ThreadPoolExecutor class
#    (mamba: returns False)
assert hasattr(futures, "ThreadPoolExecutor") == True; _ledger.append(1)

# 2) hasattr(concurrent.futures, 'ProcessPoolExecutor') — ProcessPoolExecutor class
#    (mamba: returns False)
assert hasattr(futures, "ProcessPoolExecutor") == True; _ledger.append(1)

# 3) hasattr(concurrent.futures, 'Future') — Future class
#    (mamba: returns False)
assert hasattr(futures, "Future") == True; _ledger.append(1)

# 4) hasattr(concurrent.futures, 'Executor') — Executor base class
#    (mamba: returns False)
assert hasattr(futures, "Executor") == True; _ledger.append(1)

# 5) hasattr(concurrent.futures, 'as_completed') — as_completed iterator
#    (mamba: returns False)
assert hasattr(futures, "as_completed") == True; _ledger.append(1)

# 6) hasattr(concurrent.futures, 'wait') — wait helper
#    (mamba: returns False)
assert hasattr(futures, "wait") == True; _ledger.append(1)

# 7) hasattr(concurrent.futures, 'CancelledError') — CancelledError exception
#    (mamba: returns False)
assert hasattr(futures, "CancelledError") == True; _ledger.append(1)

# 8) type(threading.Lock()).__name__ == 'lock' — lowercase 'lock' type name
#    (mamba: returns 'Lock' — capitalised type-name placeholder)
assert type(threading.Lock()).__name__ == "lock"; _ledger.append(1)

# 9) type(threading.current_thread()).__name__ == '_MainThread' — main-thread _MainThread instance
#    (mamba: returns 'Thread' — bare base-class type name)
assert type(threading.current_thread()).__name__ == "_MainThread"; _ledger.append(1)

# 10) type(threading.local()).__name__ == '_local' — TLS object type
#     (mamba: returns 'dict' — constructor degrades to plain dict)
assert type(threading.local()).__name__ == "_local"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_concurrent_futures_threading_silent {sum(_ledger)} asserts")
