# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "lru_cache_clear_resets_state"
# subject = "functools.lru_cache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.lru_cache: cache_clear zeroes hits/misses/currsize on a previously populated cache"""
import functools

@functools.lru_cache(maxsize=8)
def _square(n: int) -> int:
    return n * n


# Populate the cache so hits/misses/currsize are all non-zero.
_square(3)
_square(3)
_square(4)
_before = _square.cache_info()
assert _before.hits == 1, f"hits before clear = {_before.hits!r}"
assert _before.misses == 2, f"misses before clear = {_before.misses!r}"
assert _before.currsize == 2, f"currsize before clear = {_before.currsize!r}"

# cache_clear() drops every entry and zeroes the counters.
_square.cache_clear()
_after = _square.cache_info()
assert _after.hits == 0, f"hits after clear = {_after.hits!r}"
assert _after.misses == 0, f"misses after clear = {_after.misses!r}"
assert _after.currsize == 0, f"currsize after clear = {_after.currsize!r}"

print("lru_cache_clear_resets_state OK")
