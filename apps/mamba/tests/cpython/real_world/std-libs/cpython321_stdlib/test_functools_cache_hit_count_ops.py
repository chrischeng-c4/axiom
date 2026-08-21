# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_functools_cache_hit_count_ops"
# subject = "cpython321.test_functools_cache_hit_count_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_functools_cache_hit_count_ops.py"
# status = "filled"
# ///
"""cpython321.test_functools_cache_hit_count_ops: execute CPython 3.12 seed test_functools_cache_hit_count_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the *caching* behavior of
# `functools.cache` and the parametrized form
# `functools.lru_cache(maxsize=None)`. Surface: a function decorated
# with `@functools.cache` returns the same value across repeated
# calls *and* invokes the underlying body exactly once per distinct
# argument. The same holds for `@functools.lru_cache(maxsize=None)`,
# the canonical unbounded LRU. Cache-hit counting is established via
# a side-effect counter stored in a mutable list — counted-by-string
# to dodge the int-identity-through-closure-mutation bug carried by
# pending issue #15.
import functools
_ledger: list[int] = []

# functools.cache — call body runs exactly once per distinct arg
calls = [0]
@functools.cache
def f(x):
    calls[0] += 1
    return x * 2

r1 = f(5)
assert r1 == 10; _ledger.append(1)
r2 = f(5)
assert r2 == 10; _ledger.append(1)
assert str(calls[0]) == "1"; _ledger.append(1)
r3 = f(6)
assert r3 == 12; _ledger.append(1)
assert str(calls[0]) == "2"; _ledger.append(1)
r4 = f(6)
assert r4 == 12; _ledger.append(1)
assert str(calls[0]) == "2"; _ledger.append(1)

# functools.lru_cache(maxsize=None) — same caching guarantee
calls2 = [0]
@functools.lru_cache(maxsize=None)
def g(x):
    calls2[0] += 1
    return x + 100

s1 = g(1)
assert s1 == 101; _ledger.append(1)
s2 = g(1)
assert s2 == 101; _ledger.append(1)
assert str(calls2[0]) == "1"; _ledger.append(1)
s3 = g(2)
assert s3 == 102; _ledger.append(1)
assert str(calls2[0]) == "2"; _ledger.append(1)
s4 = g(1)
assert s4 == 101; _ledger.append(1)
assert str(calls2[0]) == "2"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_functools_cache_hit_count_ops {sum(_ledger)} asserts")
