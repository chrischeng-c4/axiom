# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "lru_cache_caches_and_counts_hits_misses"
# subject = "functools.lru_cache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.lru_cache: lru_cache memoizes by args so the body runs once per distinct key, and cache_info tracks hits/misses"""
import functools

# The body runs once per distinct argument; repeated keys are served
# from the cache and counted as hits.
_call_count = 0


@functools.lru_cache(maxsize=8)
def _square(n: int) -> int:
    global _call_count
    _call_count += 1
    return n * n


assert _square(3) == 9, "square(3)"
assert _square(3) == 9, "square(3) cached"
assert _square(4) == 16, "square(4)"
assert _square(4) == 16, "square(4) cached"
assert _call_count == 2, f"body ran {_call_count!r} times"

_info = _square.cache_info()
assert _info.hits == 2, f"hits = {_info.hits!r}"
assert _info.misses == 2, f"misses = {_info.misses!r}"
assert _info.currsize == 2, f"currsize = {_info.currsize!r}"

print("lru_cache_caches_and_counts_hits_misses OK")
