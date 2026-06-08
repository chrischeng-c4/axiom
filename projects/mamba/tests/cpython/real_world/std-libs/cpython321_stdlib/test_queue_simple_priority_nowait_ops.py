# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_queue_simple_priority_nowait_ops"
# subject = "cpython321.test_queue_simple_priority_nowait_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_queue_simple_priority_nowait_ops.py"
# status = "filled"
# ///
"""cpython321.test_queue_simple_priority_nowait_ops: execute CPython 3.12 seed test_queue_simple_priority_nowait_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 175: queue.SimpleQueue + queue.PriorityQueue (int) + nowait variants
#
# Covers queue surfaces not asserted by test_queue_ops (FIFO Queue) /
# test_queue_lifo_ops (LifoQueue):
#   - queue.SimpleQueue: empty(), qsize(), put/get FIFO ordering,
#     drain-to-empty
#   - queue.SimpleQueue: put_nowait / get_nowait non-blocking variants
#   - queue.PriorityQueue with int priorities: smallest-first invariant
#     across a full put/get drain, qsize() decrement, empty() transition
import queue

_ledger = []

# --- SimpleQueue: empty start ---
sq = queue.SimpleQueue()
assert sq.empty() == True; _ledger.append(1)
assert sq.qsize() == 0; _ledger.append(1)

# --- SimpleQueue: put / get FIFO ordering ---
sq.put("a")
sq.put("b")
sq.put("c")
assert sq.qsize() == 3; _ledger.append(1)
assert sq.empty() == False; _ledger.append(1)
assert sq.get() == "a"; _ledger.append(1)
assert sq.get() == "b"; _ledger.append(1)
assert sq.qsize() == 1; _ledger.append(1)
assert sq.get() == "c"; _ledger.append(1)
assert sq.empty() == True; _ledger.append(1)
assert sq.qsize() == 0; _ledger.append(1)

# --- SimpleQueue: int payloads roundtrip ---
sq_int = queue.SimpleQueue()
sq_int.put(1)
sq_int.put(2)
sq_int.put(3)
assert sq_int.get() == 1; _ledger.append(1)
assert sq_int.get() == 2; _ledger.append(1)
assert sq_int.get() == 3; _ledger.append(1)
assert sq_int.empty() == True; _ledger.append(1)

# --- SimpleQueue: put_nowait / get_nowait ---
sq2 = queue.SimpleQueue()
sq2.put_nowait(42)
sq2.put_nowait(43)
assert sq2.qsize() == 2; _ledger.append(1)
assert sq2.get_nowait() == 42; _ledger.append(1)
assert sq2.get_nowait() == 43; _ledger.append(1)
assert sq2.empty() == True; _ledger.append(1)

# --- SimpleQueue: re-use after drain ---
sq3 = queue.SimpleQueue()
sq3.put(10)
assert sq3.get() == 10; _ledger.append(1)
assert sq3.empty() == True; _ledger.append(1)
sq3.put(20)
assert sq3.empty() == False; _ledger.append(1)
assert sq3.qsize() == 1; _ledger.append(1)
assert sq3.get() == 20; _ledger.append(1)
assert sq3.empty() == True; _ledger.append(1)

# --- PriorityQueue (int): smallest-first ---
pq = queue.PriorityQueue()
assert pq.empty() == True; _ledger.append(1)
assert pq.qsize() == 0; _ledger.append(1)
pq.put(5)
pq.put(1)
pq.put(3)
pq.put(2)
pq.put(4)
assert pq.qsize() == 5; _ledger.append(1)
assert pq.empty() == False; _ledger.append(1)
assert pq.get() == 1; _ledger.append(1)
assert pq.get() == 2; _ledger.append(1)
assert pq.qsize() == 3; _ledger.append(1)
assert pq.get() == 3; _ledger.append(1)
assert pq.get() == 4; _ledger.append(1)
assert pq.get() == 5; _ledger.append(1)
assert pq.empty() == True; _ledger.append(1)
assert pq.qsize() == 0; _ledger.append(1)

# --- PriorityQueue (int): out-of-order insertion preserves ordering ---
pq2 = queue.PriorityQueue()
pq2.put(100)
pq2.put(-50)
pq2.put(0)
pq2.put(50)
assert pq2.get() == -50; _ledger.append(1)
assert pq2.get() == 0; _ledger.append(1)
assert pq2.get() == 50; _ledger.append(1)
assert pq2.get() == 100; _ledger.append(1)

# --- PriorityQueue (int): single element drain ---
pq3 = queue.PriorityQueue()
pq3.put(7)
assert pq3.qsize() == 1; _ledger.append(1)
assert pq3.get() == 7; _ledger.append(1)
assert pq3.empty() == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_queue_simple_priority_nowait_ops {sum(_ledger)} asserts")
