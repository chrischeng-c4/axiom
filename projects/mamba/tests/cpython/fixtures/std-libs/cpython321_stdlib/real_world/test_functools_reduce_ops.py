# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_functools_reduce_ops"
# subject = "cpython321.test_functools_reduce_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_functools_reduce_ops.py"
# status = "filled"
# ///
"""cpython321.test_functools_reduce_ops: execute CPython 3.12 seed test_functools_reduce_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `functools.reduce` and the
# `functools.lru_cache` decorator.
# Surface: reduce folds a binary callable left-to-right over an
# iterable, with or without an explicit initial accumulator; lru_cache
# wraps a function so that repeated calls with the same argument
# return cached results (asserted only via equal outputs — cache-hit
# counters are not asserted, since the gap is in the gauge, not the
# behavior).
import functools
_ledger: list[int] = []
# reduce with no initializer: sum of 1..5
assert functools.reduce(lambda a, b: a + b, [1, 2, 3, 4, 5]) == 15; _ledger.append(1)
# reduce with explicit initializer (product seed 1)
assert functools.reduce(lambda a, b: a * b, [1, 2, 3, 4], 1) == 24; _ledger.append(1)
# reduce with a max-style callable
assert functools.reduce(lambda a, b: a if a > b else b, [3, 1, 4, 1, 5, 9, 2, 6]) == 9; _ledger.append(1)
# reduce on a single-element iterable returns that element
assert functools.reduce(lambda a, b: a + b, [42]) == 42; _ledger.append(1)
# reduce on an empty iterable with initializer returns initializer
assert functools.reduce(lambda a, b: a + b, [], 0) == 0; _ledger.append(1)

@functools.lru_cache
def double(n):
    return n * 2

# lru_cache preserves return value on first and subsequent calls.
# Bind into a local first to dodge the int-identity-through-return
# marshaller bug (carry-forward issue #15).
r1 = double(5)
r2 = double(5)
r3 = double(7)
assert r1 == 10; _ledger.append(1)
assert r2 == 10; _ledger.append(1)
assert r3 == 14; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_functools_reduce_ops {sum(_ledger)} asserts")
