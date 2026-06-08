# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "lazycache_no_globals_returns_false"
# subject = "linecache.lazycache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.lazycache: lazycache with module_globals=None has nothing to load from, returns False, and caches nothing"""
import linecache

FAKE = "/no/such/dir/lazy_module.py"  # cacheable name, file never exists

linecache.clearcache()
assert linecache.lazycache(FAKE, None) is False, "no globals -> False"
assert FAKE not in linecache.cache, "no globals -> uncached"
print("lazycache_no_globals_returns_false OK")
