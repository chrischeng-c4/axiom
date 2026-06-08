# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "errors"
# case = "updatecache_missing_file_returns_empty_list"
# subject = "linecache.updatecache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.updatecache: updatecache of a non-existent file returns [] without raising"""
import linecache

linecache.clearcache()
assert linecache.updatecache("/no/such/file.py") == [], "missing file updatecache"
print("updatecache_missing_file_returns_empty_list OK")
