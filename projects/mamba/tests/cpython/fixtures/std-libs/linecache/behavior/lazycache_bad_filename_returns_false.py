# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "lazycache_bad_filename_returns_false"
# subject = "linecache.lazycache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.lazycache: lazycache rejects non-cacheable names (empty string, angle-bracketed) and returns False"""
import linecache

SRC = "line one\nline two\nline three\n"


class Loader:
    def get_source(self, name):
        return SRC


def fresh_globals():
    return {"__name__": "lazy.mod", "__loader__": Loader()}


linecache.clearcache()
assert linecache.lazycache("", fresh_globals()) is False, "empty name"
assert linecache.lazycache("<foo>", fresh_globals()) is False, "<bracket> name"
print("lazycache_bad_filename_returns_false OK")
