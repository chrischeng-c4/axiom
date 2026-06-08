# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "lazycache_registers_then_materializes"
# subject = "linecache.lazycache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.lazycache: lazycache registers a lazy 1-tuple entry for an absent file via a loader's get_source and returns True; a later getlines materializes the source"""
import linecache

SRC = "line one\nline two\nline three\n"
FAKE = "/no/such/dir/lazy_module.py"  # cacheable name, file never exists


class Loader:
    def get_source(self, name):
        return SRC


def fresh_globals():
    return {"__name__": "lazy.mod", "__loader__": Loader()}


linecache.clearcache()
assert linecache.lazycache(FAKE, fresh_globals()) is True, "lazy registered"
assert len(linecache.cache[FAKE]) == 1, "entry is lazy 1-tuple"
# getlines materializes the source via the loader.
assert linecache.getlines(FAKE) == ["line one\n", "line two\n", "line three\n"], "materialized"
print("lazycache_registers_then_materializes OK")
