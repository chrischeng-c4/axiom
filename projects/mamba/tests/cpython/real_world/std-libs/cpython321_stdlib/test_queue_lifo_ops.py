# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_queue_lifo_ops"
# subject = "cpython321.test_queue_lifo_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_queue_lifo_ops.py"
# status = "filled"
# ///
"""cpython321.test_queue_lifo_ops: execute CPython 3.12 seed test_queue_lifo_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `queue.LifoQueue` — the
# stack-ordered (last-in, first-out) counterpart to `queue.Queue`.
# Surface: empty start; `qsize()` returns 0 on empty; `put`/`get`
# follow LIFO order so the most recently inserted item is yielded
# first; `qsize` decrements as items are drained; the queue returns to
# empty when drained. Both integer and string payloads round-trip; a
# single-element put/get pair returns the queue to the empty state.
import queue
_ledger: list[int] = []

lq = queue.LifoQueue()

# Empty queue starts empty
assert lq.empty() == True; _ledger.append(1)
assert lq.qsize() == 0; _ledger.append(1)

# Put a sequence of items
lq.put(1)
lq.put(2)
lq.put(3)
assert lq.qsize() == 3; _ledger.append(1)
assert lq.empty() == False; _ledger.append(1)

# Get returns in LIFO order — last in, first out
assert lq.get() == 3; _ledger.append(1)
assert lq.get() == 2; _ledger.append(1)
assert lq.qsize() == 1; _ledger.append(1)
assert lq.get() == 1; _ledger.append(1)
assert lq.empty() == True; _ledger.append(1)
assert lq.qsize() == 0; _ledger.append(1)

# String payloads round-trip through LIFO
lq2 = queue.LifoQueue()
lq2.put("a")
lq2.put("b")
assert lq2.get() == "b"; _ledger.append(1)
assert lq2.get() == "a"; _ledger.append(1)

# Single put/get returns to empty
lq3 = queue.LifoQueue()
lq3.put(99)
lq3.get()
assert lq3.empty() == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_queue_lifo_ops {sum(_ledger)} asserts")
