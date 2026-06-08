# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "lru_cache_bare_and_unbounded_forms"
# subject = "functools.lru_cache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.lru_cache: the bare @lru_cache form and @lru_cache(maxsize=None) both cache; unbounded grows currsize with each distinct key"""
import functools

# Bare decorator form: `@lru_cache` with no parens still memoizes and
# defaults to maxsize=128.
_bare_calls = 0


@functools.lru_cache
def _double(n: int) -> int:
    global _bare_calls
    _bare_calls += 1
    return n * 2


assert _double(5) == 10, "double(5)"
assert _double(5) == 10, "double(5) cached"
assert _double(10) == 20, "double(10)"
assert _bare_calls == 2, f"bare body ran {_bare_calls!r} times"

_bare_info = _double.cache_info()
assert _bare_info.hits == 1, f"bare hits = {_bare_info.hits!r}"
assert _bare_info.misses == 2, f"bare misses = {_bare_info.misses!r}"
assert _bare_info.maxsize == 128, f"bare maxsize = {_bare_info.maxsize!r}"


# Unbounded form: maxsize=None never evicts, so currsize grows with each
# distinct key.
@functools.lru_cache(maxsize=None)
def _triple(n: int) -> int:
    return n * 3


for i in range(20):
    _triple(i)

_un_info = _triple.cache_info()
assert _un_info.maxsize is None, f"unbounded maxsize = {_un_info.maxsize!r}"
assert _un_info.currsize == 20, f"unbounded currsize = {_un_info.currsize!r}"

print("lru_cache_bare_and_unbounded_forms OK")
