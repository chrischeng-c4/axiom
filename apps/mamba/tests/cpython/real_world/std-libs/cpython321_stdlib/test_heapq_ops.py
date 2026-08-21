# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_heapq_ops"
# subject = "cpython321.test_heapq_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_heapq_ops.py"
# status = "filled"
# ///
"""cpython321.test_heapq_ops: execute CPython 3.12 seed test_heapq_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `heapq` stdlib module.
# Surface: heapify, heappush, heappop yielding ascending order,
# nsmallest/nlargest helpers.
# Companion to stub/test_heapq.py — vendored unittest seed.
import heapq
_ledger: list[int] = []

h = [3, 1, 4, 1, 5, 9, 2, 6]
heapq.heapify(h)
assert heapq.heappop(h) == 1; _ledger.append(1)
assert heapq.heappop(h) == 1; _ledger.append(1)
assert heapq.heappop(h) == 2; _ledger.append(1)

heapq.heappush(h, 0)
assert heapq.heappop(h) == 0; _ledger.append(1)

assert heapq.nsmallest(3, [5, 1, 4, 1, 3, 2]) == [1, 1, 2]; _ledger.append(1)
assert heapq.nlargest(3, [5, 1, 4, 1, 3, 2]) == [5, 4, 3]; _ledger.append(1)

# Pop everything from a fresh heap yields ascending order
src = [9, 3, 7, 1, 5, 2, 8, 4, 6]
heapq.heapify(src)
sorted_out: list[int] = []
while src:
    sorted_out.append(heapq.heappop(src))
assert sorted_out == [1, 2, 3, 4, 5, 6, 7, 8, 9]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_heapq_ops {sum(_ledger)} asserts")
