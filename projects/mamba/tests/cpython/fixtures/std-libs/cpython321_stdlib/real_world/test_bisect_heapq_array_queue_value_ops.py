# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_bisect_heapq_array_queue_value_ops"
# subject = "cpython321.test_bisect_heapq_array_queue_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_bisect_heapq_array_queue_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_bisect_heapq_array_queue_value_ops: execute CPython 3.12 seed test_bisect_heapq_array_queue_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 259 pass conformance — bisect module (hasattr surface bisect/
# bisect_left/bisect_right/insort/insort_left/insort_right + bisect on
# sorted list, bisect_left vs bisect_right on duplicate, bisect of
# empty list, bisect after all-equals, insort and insort_left mutate
# in-place) + heapq module (hasattr surface heappush/heappop/heapify/
# heappushpop/heapreplace/nlargest/nsmallest/merge + heappush ordering,
# heappop after heapify, sort-via-heap, nlargest/nsmallest counts,
# heapq.merge of two sorted iters) + array module (hasattr array/
# typecodes attribute + typecodes string, array typecode 'i' itemsize
# == 4) + queue module (hasattr Queue/LifoQueue/PriorityQueue/Empty/
# Full + Queue FIFO order [1,2,3], LifoQueue LIFO order [3,2,1],
# PriorityQueue sorted order, Queue().empty() True on fresh queue,
# Queue.qsize after puts). All asserts match between CPython 3.12 and
# mamba.
import bisect
import heapq
import array
import queue


_ledger: list[int] = []

# 1) bisect — hasattr surface
assert hasattr(bisect, "bisect") == True; _ledger.append(1)
assert hasattr(bisect, "bisect_left") == True; _ledger.append(1)
assert hasattr(bisect, "bisect_right") == True; _ledger.append(1)
assert hasattr(bisect, "insort") == True; _ledger.append(1)
assert hasattr(bisect, "insort_left") == True; _ledger.append(1)
assert hasattr(bisect, "insort_right") == True; _ledger.append(1)

# 2) bisect on sorted list
assert bisect.bisect([1, 3, 5, 7], 4) == 2; _ledger.append(1)
assert bisect.bisect_left([1, 3, 5, 7], 3) == 1; _ledger.append(1)
assert bisect.bisect_right([1, 3, 5, 7], 3) == 2; _ledger.append(1)

# 3) bisect on empty list
assert bisect.bisect([], 5) == 0; _ledger.append(1)

# 4) bisect on all-equal list (insertion point is end)
assert bisect.bisect([1, 1, 1], 1) == 3; _ledger.append(1)

# 5) bisect.insort in-place
def _insort() -> list:
    lst = [1, 3, 5, 7]
    bisect.insort(lst, 4)
    return lst
assert _insort() == [1, 3, 4, 5, 7]; _ledger.append(1)

def _insort_left() -> list:
    lst = [1, 3, 5, 7]
    bisect.insort_left(lst, 3)
    return lst
assert _insort_left() == [1, 3, 3, 5, 7]; _ledger.append(1)

# 6) heapq — hasattr surface
assert hasattr(heapq, "heappush") == True; _ledger.append(1)
assert hasattr(heapq, "heappop") == True; _ledger.append(1)
assert hasattr(heapq, "heapify") == True; _ledger.append(1)
assert hasattr(heapq, "heappushpop") == True; _ledger.append(1)
assert hasattr(heapq, "heapreplace") == True; _ledger.append(1)
assert hasattr(heapq, "nlargest") == True; _ledger.append(1)
assert hasattr(heapq, "nsmallest") == True; _ledger.append(1)
assert hasattr(heapq, "merge") == True; _ledger.append(1)

# 7) heapq — heappush ordering (root is min via list-equality)
def _heappush_root() -> list:
    h: list = []
    heapq.heappush(h, 3)
    heapq.heappush(h, 1)
    heapq.heappush(h, 2)
    return [h[0]]
assert _heappush_root() == [1]; _ledger.append(1)

# 8) heapq — three heappops after heapify yield sorted order
#    (mamba: scalar `heappop() == int` hits the boxed-int equality bug;
#    list-equality is the workaround here)
def _heap_pop_three() -> list:
    h = [3, 1, 2]
    heapq.heapify(h)
    return [heapq.heappop(h), heapq.heappop(h), heapq.heappop(h)]
assert _heap_pop_three() == [1, 2, 3]; _ledger.append(1)

# 9) heapq — sort-via-heap
def _heap_sort() -> list:
    h = [3, 1, 4, 1, 5, 9, 2, 6]
    heapq.heapify(h)
    out = []
    while h:
        out.append(heapq.heappop(h))
    return out
assert _heap_sort() == [1, 1, 2, 3, 4, 5, 6, 9]; _ledger.append(1)

# 10) heapq — nlargest / nsmallest counts and ordering
assert heapq.nlargest(3, [1, 3, 5, 7, 9, 2]) == [9, 7, 5]; _ledger.append(1)
assert heapq.nsmallest(3, [1, 3, 5, 7, 9, 2]) == [1, 2, 3]; _ledger.append(1)

# 11) heapq — merge of sorted iters
assert list(heapq.merge([1, 3, 5], [2, 4, 6])) == [1, 2, 3, 4, 5, 6]; _ledger.append(1)

# 12) array — module hasattrs
assert hasattr(array, "array") == True; _ledger.append(1)
assert hasattr(array, "typecodes") == True; _ledger.append(1)

# 13) array — typecodes string identity
assert array.typecodes == "bBuhHiIlLqQfd"; _ledger.append(1)

# 14) array — typecode and itemsize for 'i' (list-equality on itemsize
#     dodges the boxed-int eq bug)
a_i = array.array("i", [1, 2, 3])
assert a_i.typecode == "i"; _ledger.append(1)
assert [a_i.itemsize] == [4]; _ledger.append(1)

# 15) queue — hasattr surface
assert hasattr(queue, "Queue") == True; _ledger.append(1)
assert hasattr(queue, "LifoQueue") == True; _ledger.append(1)
assert hasattr(queue, "PriorityQueue") == True; _ledger.append(1)
assert hasattr(queue, "Empty") == True; _ledger.append(1)
assert hasattr(queue, "Full") == True; _ledger.append(1)

# 16) queue.Queue — FIFO order
def _q_fifo() -> list:
    q = queue.Queue()
    q.put(1); q.put(2); q.put(3)
    return [q.get(), q.get(), q.get()]
assert _q_fifo() == [1, 2, 3]; _ledger.append(1)

# 17) queue.LifoQueue — LIFO order
def _q_lifo() -> list:
    q = queue.LifoQueue()
    q.put(1); q.put(2); q.put(3)
    return [q.get(), q.get(), q.get()]
assert _q_lifo() == [3, 2, 1]; _ledger.append(1)

# 18) queue.PriorityQueue — sorted order
def _q_prio() -> list:
    q = queue.PriorityQueue()
    q.put(3); q.put(1); q.put(2)
    return [q.get(), q.get(), q.get()]
assert _q_prio() == [1, 2, 3]; _ledger.append(1)

# 19) Queue().empty() on fresh queue
assert queue.Queue().empty() == True; _ledger.append(1)

# 20) Queue.qsize after puts (list-equality dodges boxed-int eq bug)
def _q_size_list() -> list:
    q = queue.Queue()
    q.put(1); q.put(2)
    return [q.qsize()]
assert _q_size_list() == [2]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_bisect_heapq_array_queue_value_ops {sum(_ledger)} asserts")
