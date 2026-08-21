# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_heapq_bisect_queue_threading_contextvars_value_ops"
# subject = "cpython321.test_heapq_bisect_queue_threading_contextvars_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_heapq_bisect_queue_threading_contextvars_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_heapq_bisect_queue_threading_contextvars_value_ops: execute CPython 3.12 seed test_heapq_bisect_queue_threading_contextvars_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 287 pass conformance — heapq module (hasattr heappush/
# heappop/heapify/heapreplace/heappushpop/nlargest/nsmallest/merge
# + heappushpop returns min + heapreplace returns root + nlargest/
# nsmallest selection) + bisect module (hasattr bisect_left/
# bisect_right/bisect/insort_left/insort_right/insort + bisect_left
# 3/bisect_right 3/bisect 4 + insort grows list) + queue module
# (hasattr Queue/LifoQueue/PriorityQueue/SimpleQueue/Empty/Full) +
# threading module (hasattr Thread/Lock/RLock/Condition/Semaphore/
# BoundedSemaphore/Event/Barrier/Timer/current_thread/main_thread/
# active_count/enumerate/get_ident/local + get_ident is int +
# active_count >= 1) + _thread module (hasattr allocate_lock/
# get_ident/start_new_thread/error) + contextvars module (hasattr
# ContextVar/Context/copy_context/Token) + reprlib module (hasattr
# Repr/repr/recursive_repr + repr([]) == '[]') + pprint module
# (hasattr pprint/pformat + pformat({}) == '{}').
# All asserts match between CPython 3.12 and mamba.
import heapq
import bisect
import queue
import threading
import _thread
import contextvars
import reprlib
import pprint


_ledger: list[int] = []

# 1) heapq — hasattr surface
assert hasattr(heapq, "heappush") == True; _ledger.append(1)
assert hasattr(heapq, "heappop") == True; _ledger.append(1)
assert hasattr(heapq, "heapify") == True; _ledger.append(1)
assert hasattr(heapq, "heapreplace") == True; _ledger.append(1)
assert hasattr(heapq, "heappushpop") == True; _ledger.append(1)
assert hasattr(heapq, "nlargest") == True; _ledger.append(1)
assert hasattr(heapq, "nsmallest") == True; _ledger.append(1)
assert hasattr(heapq, "merge") == True; _ledger.append(1)

# 2) heapq — value contracts
assert heapq.heappushpop([1, 2, 3], 0) == 0; _ledger.append(1)
assert heapq.heapreplace([1, 2, 3], 0) == 1; _ledger.append(1)
assert heapq.nlargest(3, [1, 5, 3, 7, 2, 8]) == [8, 7, 5]; _ledger.append(1)
assert heapq.nsmallest(3, [1, 5, 3, 7, 2, 8]) == [1, 2, 3]; _ledger.append(1)

# 3) bisect — hasattr surface
assert hasattr(bisect, "bisect_left") == True; _ledger.append(1)
assert hasattr(bisect, "bisect_right") == True; _ledger.append(1)
assert hasattr(bisect, "bisect") == True; _ledger.append(1)
assert hasattr(bisect, "insort_left") == True; _ledger.append(1)
assert hasattr(bisect, "insort_right") == True; _ledger.append(1)
assert hasattr(bisect, "insort") == True; _ledger.append(1)

# 4) bisect — value contracts
assert bisect.bisect_left([1, 3, 5, 7], 3) == 1; _ledger.append(1)
assert bisect.bisect_right([1, 3, 5, 7], 3) == 2; _ledger.append(1)
assert bisect.bisect([1, 3, 5, 7], 4) == 2; _ledger.append(1)
_arr = [1, 3, 5, 7]
bisect.insort(_arr, 4)
assert _arr == [1, 3, 4, 5, 7]; _ledger.append(1)

# 5) queue — hasattr surface
assert hasattr(queue, "Queue") == True; _ledger.append(1)
assert hasattr(queue, "LifoQueue") == True; _ledger.append(1)
assert hasattr(queue, "PriorityQueue") == True; _ledger.append(1)
assert hasattr(queue, "SimpleQueue") == True; _ledger.append(1)
assert hasattr(queue, "Empty") == True; _ledger.append(1)
assert hasattr(queue, "Full") == True; _ledger.append(1)

# 6) threading — hasattr handler surface
assert hasattr(threading, "Thread") == True; _ledger.append(1)
assert hasattr(threading, "Lock") == True; _ledger.append(1)
assert hasattr(threading, "RLock") == True; _ledger.append(1)
assert hasattr(threading, "Condition") == True; _ledger.append(1)
assert hasattr(threading, "Semaphore") == True; _ledger.append(1)
assert hasattr(threading, "BoundedSemaphore") == True; _ledger.append(1)
assert hasattr(threading, "Event") == True; _ledger.append(1)
assert hasattr(threading, "Barrier") == True; _ledger.append(1)
assert hasattr(threading, "Timer") == True; _ledger.append(1)
assert hasattr(threading, "current_thread") == True; _ledger.append(1)
assert hasattr(threading, "main_thread") == True; _ledger.append(1)
assert hasattr(threading, "active_count") == True; _ledger.append(1)
assert hasattr(threading, "enumerate") == True; _ledger.append(1)
assert hasattr(threading, "get_ident") == True; _ledger.append(1)
assert hasattr(threading, "local") == True; _ledger.append(1)

# 7) threading — value contracts
assert isinstance(threading.get_ident(), int) == True; _ledger.append(1)
assert (threading.active_count() >= 1) == True; _ledger.append(1)

# 8) _thread — hasattr surface
assert hasattr(_thread, "allocate_lock") == True; _ledger.append(1)
assert hasattr(_thread, "get_ident") == True; _ledger.append(1)
assert hasattr(_thread, "start_new_thread") == True; _ledger.append(1)
assert hasattr(_thread, "error") == True; _ledger.append(1)

# 9) contextvars — hasattr surface
assert hasattr(contextvars, "ContextVar") == True; _ledger.append(1)
assert hasattr(contextvars, "Context") == True; _ledger.append(1)
assert hasattr(contextvars, "copy_context") == True; _ledger.append(1)
assert hasattr(contextvars, "Token") == True; _ledger.append(1)

# 10) reprlib — hasattr + behavior
assert hasattr(reprlib, "Repr") == True; _ledger.append(1)
assert hasattr(reprlib, "repr") == True; _ledger.append(1)
assert hasattr(reprlib, "recursive_repr") == True; _ledger.append(1)
assert reprlib.repr([]) == "[]"; _ledger.append(1)

# 11) pprint — hasattr + minimal behavior
assert hasattr(pprint, "pprint") == True; _ledger.append(1)
assert hasattr(pprint, "pformat") == True; _ledger.append(1)
assert pprint.pformat({}) == "{}"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_heapq_bisect_queue_threading_contextvars_value_ops {sum(_ledger)} asserts")
