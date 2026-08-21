# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `type(queue.Queue()).__name__` (the
# documented "Queue() returns a Queue instance" — mamba returns
# 'int' — Queue is an opaque int handle), `type(threading.Lock()).
# __name__` (the documented "Lock() returns a 'lock' instance" —
# mamba returns 'Lock' — class name not lowercased), `type(threading
# .current_thread()).__name__` (the documented "current_thread()
# returns a '_MainThread' subclass" — mamba returns 'Thread' — no
# main subclass distinction), `type(heapq.merge([1],[2])).__name__`
# (the documented "merge returns a 'generator'" — mamba returns
# 'list' — eager materialization), `type(contextvars.ContextVar
# ('x')).__name__` (the documented "ContextVar() returns a
# 'ContextVar'" — mamba returns 'str' — opaque str handle), `type
# (contextvars.copy_context()).__name__` (the documented
# "copy_context() returns a 'Context'" — mamba returns 'str' —
# opaque str handle), `hasattr(reprlib, 'aRepr')` (the documented
# "reprlib exposes the default aRepr instance" — mamba returns
# False), `hasattr(pprint, 'PrettyPrinter')` (the documented
# "pprint exposes the PrettyPrinter class" — mamba returns False),
# `hasattr(pprint, 'isreadable')` (the documented "pprint exposes
# the isreadable helper" — mamba returns False), and `pprint.
# pformat([1, 2, 3]) == '[1, 2, 3]'` (the documented "small lists
# pformat single-line" — mamba returns '[\\n 1,\\n 2,\\n 3\\n]' —
# always multi-line).
# Ten-pack pinned to atomic 287.
#
# Behavioral edges that CONFORM on mamba (heapq — hasattr heappush/
# heappop/heapify/heapreplace/heappushpop/nlargest/nsmallest/merge +
# behavior. bisect — hasattr bisect_left/bisect_right/bisect/
# insort_left/insort_right/insort + behavior. queue — hasattr Queue/
# LifoQueue/PriorityQueue/SimpleQueue/Empty/Full. threading —
# hasattr Thread/Lock/RLock/Condition/Semaphore/BoundedSemaphore/
# Event/Barrier/Timer/current_thread/main_thread/active_count/
# enumerate/get_ident/local + get_ident int + active_count >= 1.
# _thread — hasattr allocate_lock/get_ident/start_new_thread/error.
# contextvars — hasattr ContextVar/Context/copy_context/Token.
# reprlib — hasattr Repr/repr/recursive_repr + repr([]). pprint —
# hasattr pprint/pformat + pformat({})) are covered in the matching
# pass fixture `test_heapq_bisect_queue_threading_contextvars_
# value_ops`.
import queue
import threading
import heapq
import contextvars
import reprlib
import pprint


_ledger: list[int] = []

# 1) type(queue.Queue()).__name__ == 'Queue' — Queue instance class
#    (mamba: returns 'int' — Queue is an opaque int handle)
assert type(queue.Queue()).__name__ == "Queue"; _ledger.append(1)

# 2) type(threading.Lock()).__name__ == 'lock' — Lock instance class
#    (mamba: returns 'Lock' — class name not lowercased)
assert type(threading.Lock()).__name__ == "lock"; _ledger.append(1)

# 3) type(threading.current_thread()).__name__ == '_MainThread' — main subclass
#    (mamba: returns 'Thread' — no main subclass distinction)
assert type(threading.current_thread()).__name__ == "_MainThread"; _ledger.append(1)

# 4) type(heapq.merge([1], [2])).__name__ == 'generator' — lazy iterator
#    (mamba: returns 'list' — eager materialization)
assert type(heapq.merge([1], [2])).__name__ == "generator"; _ledger.append(1)

# 5) type(contextvars.ContextVar('x')).__name__ == 'ContextVar' — class
#    (mamba: returns 'str' — opaque str handle)
assert type(contextvars.ContextVar("x")).__name__ == "ContextVar"; _ledger.append(1)

# 6) type(contextvars.copy_context()).__name__ == 'Context' — class
#    (mamba: returns 'str' — opaque str handle)
assert type(contextvars.copy_context()).__name__ == "Context"; _ledger.append(1)

# 7) hasattr(reprlib, 'aRepr') — default Repr singleton
#    (mamba: returns False)
assert hasattr(reprlib, "aRepr") == True; _ledger.append(1)

# 8) hasattr(pprint, 'PrettyPrinter') — PrettyPrinter class
#    (mamba: returns False)
assert hasattr(pprint, "PrettyPrinter") == True; _ledger.append(1)

# 9) hasattr(pprint, 'isreadable') — readability check helper
#    (mamba: returns False)
assert hasattr(pprint, "isreadable") == True; _ledger.append(1)

# 10) pprint.pformat([1, 2, 3]) == '[1, 2, 3]' — single-line small list
#     (mamba: returns '[\n 1,\n 2,\n 3\n]' — always multi-line)
assert pprint.pformat([1, 2, 3]) == "[1, 2, 3]"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_queue_threading_contextvars_pprint_silent {sum(_ledger)} asserts")
