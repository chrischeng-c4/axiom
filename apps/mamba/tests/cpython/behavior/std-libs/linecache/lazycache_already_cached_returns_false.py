# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "lazycache_already_cached_returns_false"
# subject = "linecache.lazycache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.lazycache: lazycache leaves a fully-cached 4-tuple entry untouched and returns False"""
import linecache

SRC = "line one\nline two\nline three\n"
FAKE = "/no/such/dir/lazy_module.py"  # cacheable name, file never exists


class Loader:
    def get_source(self, name):
        return SRC


def fresh_globals():
    return {"__name__": "lazy.mod", "__loader__": Loader()}


linecache.clearcache()
before = linecache.getlines(FAKE, fresh_globals())
assert before == ["line one\n", "line two\n", "line three\n"], "pre-cache"
assert linecache.lazycache(FAKE, fresh_globals()) is False, "already cached -> False"
assert len(linecache.cache[FAKE]) == 4, "stays full 4-tuple"
print("lazycache_already_cached_returns_false OK")
