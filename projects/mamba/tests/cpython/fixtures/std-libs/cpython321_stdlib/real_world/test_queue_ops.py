# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_queue_ops"
# subject = "cpython321.test_queue_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_queue_ops.py"
# status = "filled"
# ///
"""cpython321.test_queue_ops: execute CPython 3.12 seed test_queue_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `queue.Queue` FIFO surface.
# Surface: put/get FIFO ordering, qsize after a sequence of puts,
# empty() flag transitions, drain to empty preserves order.
# Companion to stub/test_queue.py — vendored unittest seed.
import queue
_ledger: list[int] = []
q = queue.Queue()
# Empty queue starts empty
assert q.empty(); _ledger.append(1)
assert q.qsize() == 0; _ledger.append(1)
# Put a sequence of items
q.put(1)
q.put(2)
q.put(3)
assert q.qsize() == 3; _ledger.append(1)
assert not q.empty(); _ledger.append(1)
# Get returns in FIFO order
assert q.get() == 1; _ledger.append(1)
assert q.get() == 2; _ledger.append(1)
assert q.qsize() == 1; _ledger.append(1)
assert not q.empty(); _ledger.append(1)
# Final get drains to empty
assert q.get() == 3; _ledger.append(1)
assert q.empty(); _ledger.append(1)
assert q.qsize() == 0; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_queue_ops {sum(_ledger)} asserts")
