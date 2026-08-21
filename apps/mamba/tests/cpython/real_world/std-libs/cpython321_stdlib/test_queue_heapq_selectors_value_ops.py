# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_queue_heapq_selectors_value_ops"
# subject = "cpython321.test_queue_heapq_selectors_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_queue_heapq_selectors_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_queue_heapq_selectors_value_ops: execute CPython 3.12 seed test_queue_heapq_selectors_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `queue` / `heapq` / `selectors` three-pack pinned to atomic
# 199: `queue` (the documented full module-level class /
# exception identifier hasattr surface — `Queue` /
# `LifoQueue` / `PriorityQueue` / `SimpleQueue` / `Empty` /
# `Full` + the documented Queue put / qsize / get round-trip
# value contract), `heapq` (the documented full module-level
# helper hasattr surface — `heappush` / `heappop` /
# `heapify` / `heapreplace` / `heappushpop` / `nlargest` /
# `nsmallest` / `merge` + the documented heapify[0] == 1
# minimum-extraction + nlargest / nsmallest list-returning
# value contract), and `selectors` (the documented partial
# module-level selector / constant identifier hasattr
# surface — `DefaultSelector` / `SelectSelector` /
# `PollSelector` / `KqueueSelector` / `BaseSelector` /
# `SelectorKey` / `EVENT_READ` / `EVENT_WRITE` + the
# documented EVENT_READ == 1 / EVENT_WRITE == 2 integer-
# value contract).
#
# Behavioral edges that DIVERGE on mamba
# (type(queue.Queue()).__name__ collapses to "int" on mamba
# via the integer-handle pattern instead of "Queue",
# hasattr(sched, "scheduler") / "Event" all False on mamba,
# hasattr(mmap, "mmap") / "ACCESS_READ" / "ACCESS_WRITE" /
# "ACCESS_COPY" / "ACCESS_DEFAULT" / "PROT_READ" /
# "PROT_WRITE" / "PROT_EXEC" / "MAP_SHARED" / "MAP_PRIVATE"
# / "MAP_ANON" / "MAP_ANONYMOUS" / "PAGESIZE" /
# "ALLOCATIONGRANULARITY" all False on mamba) are covered
# in the matching spec fixture `lang_queue_sched_mmap_silent`.
import queue
import heapq
import selectors


_ledger: list[int] = []

# 1) queue — full module hasattr surface
#    (Queue instance class identity collapses to "int" on
#    mamba via the integer-handle pattern — moved to spec)
assert hasattr(queue, "Queue") == True; _ledger.append(1)
assert hasattr(queue, "LifoQueue") == True; _ledger.append(1)
assert hasattr(queue, "PriorityQueue") == True; _ledger.append(1)
assert hasattr(queue, "SimpleQueue") == True; _ledger.append(1)
assert hasattr(queue, "Empty") == True; _ledger.append(1)
assert hasattr(queue, "Full") == True; _ledger.append(1)

# 2) queue.Queue — put / qsize / get round-trip value contract
_q = queue.Queue()
_q.put(1); _q.put(2); _q.put(3)
assert _q.qsize() == 3; _ledger.append(1)
assert _q.get() == 1; _ledger.append(1)
assert _q.get() == 2; _ledger.append(1)
assert _q.get() == 3; _ledger.append(1)

# 3) heapq — full module hasattr surface
assert hasattr(heapq, "heappush") == True; _ledger.append(1)
assert hasattr(heapq, "heappop") == True; _ledger.append(1)
assert hasattr(heapq, "heapify") == True; _ledger.append(1)
assert hasattr(heapq, "heapreplace") == True; _ledger.append(1)
assert hasattr(heapq, "heappushpop") == True; _ledger.append(1)
assert hasattr(heapq, "nlargest") == True; _ledger.append(1)
assert hasattr(heapq, "nsmallest") == True; _ledger.append(1)
assert hasattr(heapq, "merge") == True; _ledger.append(1)

# 4) heapq — heapify minimum-extraction + nlargest / nsmallest
_lst = [3, 1, 4, 1, 5, 9, 2, 6]
heapq.heapify(_lst)
assert _lst[0] == 1; _ledger.append(1)
assert heapq.nlargest(3, [3, 1, 4, 1, 5, 9, 2, 6]) == [9, 6, 5]; _ledger.append(1)
assert heapq.nsmallest(3, [3, 1, 4, 1, 5, 9, 2, 6]) == [1, 1, 2]; _ledger.append(1)

# 5) selectors — partial module hasattr surface
assert hasattr(selectors, "DefaultSelector") == True; _ledger.append(1)
assert hasattr(selectors, "SelectSelector") == True; _ledger.append(1)
assert hasattr(selectors, "PollSelector") == True; _ledger.append(1)
assert hasattr(selectors, "KqueueSelector") == True; _ledger.append(1)
assert hasattr(selectors, "BaseSelector") == True; _ledger.append(1)
assert hasattr(selectors, "SelectorKey") == True; _ledger.append(1)
assert hasattr(selectors, "EVENT_READ") == True; _ledger.append(1)
assert hasattr(selectors, "EVENT_WRITE") == True; _ledger.append(1)

# 6) selectors — integer-value contract
assert selectors.EVENT_READ == 1; _ledger.append(1)
assert selectors.EVENT_WRITE == 2; _ledger.append(1)

# NB: type(queue.Queue()).__name__ collapses to "int" on mamba
# via the integer-handle pattern instead of "Queue",
# hasattr(sched, "scheduler") / "Event" all False on mamba,
# hasattr(mmap, "mmap") / "ACCESS_READ" / "ACCESS_WRITE" /
# "ACCESS_COPY" / "ACCESS_DEFAULT" / "PROT_READ" /
# "PROT_WRITE" / "PROT_EXEC" / "MAP_SHARED" / "MAP_PRIVATE"
# / "MAP_ANON" / "MAP_ANONYMOUS" / "PAGESIZE" /
# "ALLOCATIONGRANULARITY" all False on mamba — all DIVERGE
# on mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_queue_heapq_selectors_value_ops {sum(_ledger)} asserts")
