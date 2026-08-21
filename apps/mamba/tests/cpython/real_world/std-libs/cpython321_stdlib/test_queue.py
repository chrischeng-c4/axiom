# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_queue"
# subject = "cpython321.test_queue"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_queue.py"
# status = "filled"
# ///
"""cpython321.test_queue: execute CPython 3.12 seed test_queue"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
import queue

_ledger: list[int] = []

q = queue.Queue()
q.put(1)
q.put(2)
q.put(3)
assert q.qsize() == 3, "FIFO qsize after 3 puts"
_ledger.append(1)

assert q.empty() == False, "FIFO not empty after puts"
_ledger.append(1)

assert q.get() == 1, "FIFO get 1st item"
_ledger.append(1)

assert q.get() == 2, "FIFO get 2nd item"
_ledger.append(1)

assert q.get() == 3, "FIFO get 3rd item"
_ledger.append(1)

assert q.empty() == True, "FIFO empty after all gets"
_ledger.append(1)

lq = queue.LifoQueue()
lq.put(1)
lq.put(2)
lq.put(3)
assert lq.get() == 3, "LIFO returns most recent"
_ledger.append(1)

assert lq.get() == 2, "LIFO returns second most recent"
_ledger.append(1)

assert lq.get() == 1, "LIFO returns least recent last"
_ledger.append(1)

pq = queue.PriorityQueue()
pq.put(3)
pq.put(1)
pq.put(2)
assert pq.get() == 1, "PriorityQueue returns smallest first"
_ledger.append(1)

assert pq.get() == 2, "PriorityQueue returns next smallest"
_ledger.append(1)

assert pq.get() == 3, "PriorityQueue returns largest last"
_ledger.append(1)

bq = queue.Queue(maxsize=2)
bq.put("a")
bq.put("b")
assert bq.qsize() == 2, "bounded Queue accepts maxsize items"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: queue {sum(_ledger)} asserts")
