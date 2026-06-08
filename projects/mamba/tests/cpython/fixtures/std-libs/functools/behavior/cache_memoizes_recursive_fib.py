# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "cache_memoizes_recursive_fib"
# subject = "functools.cache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.cache: functools.cache memoizes a recursive fibonacci and a multi-arg adder so repeated keys return the cached value"""
import functools

# Recursive fibonacci: the body runs once per distinct n (12 distinct
# keys for fib(10): 0..10 plus the top call) thanks to memoization.
_fib_calls = 0


@functools.cache
def _fib(n: int) -> int:
    global _fib_calls
    _fib_calls += 1
    return n if n < 2 else _fib(n - 1) + _fib(n - 2)


assert _fib(10) == 55, f"fib(10) = {_fib(10)!r}"
assert _fib_calls == 11, f"fib body ran {_fib_calls!r} times"
assert _fib(10) == 55, "fib(10) cached"

_info = _fib.cache_info()
assert _info.misses == 11, f"fib misses = {_info.misses!r}"
assert _info.currsize == 11, f"fib currsize = {_info.currsize!r}"


# Multi-arg adder: repeated (a, b) keys hit the cache; the body runs once
# per distinct argument tuple.
_add_calls = 0


@functools.cache
def _add(a: int, b: int) -> int:
    global _add_calls
    _add_calls += 1
    return a + b


assert _add(2, 3) == 5, "add(2,3)"
assert _add(2, 3) == 5, "add(2,3) cached"
assert _add(4, 5) == 9, "add(4,5)"
assert _add_calls == 2, f"add body ran {_add_calls!r} times"

_ainfo = _add.cache_info()
assert _ainfo.hits == 1, f"add hits = {_ainfo.hits!r}"
assert _ainfo.misses == 2, f"add misses = {_ainfo.misses!r}"
assert _ainfo.currsize == 2, f"add currsize = {_ainfo.currsize!r}"

print("cache_memoizes_recursive_fib OK")
