# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "lru_cache_maxsize_evicts_lru"
# subject = "functools.lru_cache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.lru_cache: lru_cache(maxsize=N) bounds currsize at N and evicts the least-recently-used entry"""
import functools

# maxsize=2 holds at most two entries. square(3) evicts the LRU entry
# square(1), so the later square(1) is a miss that recomputes the body.
_calls = 0


@functools.lru_cache(maxsize=2)
def _square(n: int) -> int:
    global _calls
    _calls += 1
    return n * n


assert _square(1) == 1, "square(1)"
assert _square(2) == 4, "square(2)"
assert _square(3) == 9, "square(3) evicts square(1)"
assert _square(1) == 1, "square(1) recomputed after eviction"
assert _calls == 4, f"body ran {_calls!r} times (no hits)"

_info = _square.cache_info()
assert _info.maxsize == 2, f"maxsize = {_info.maxsize!r}"
assert _info.currsize == 2, f"currsize bounded at 2 = {_info.currsize!r}"
assert _info.hits == 0, f"hits = {_info.hits!r}"
assert _info.misses == 4, f"misses = {_info.misses!r}"

print("lru_cache_maxsize_evicts_lru OK")
