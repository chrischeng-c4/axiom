# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "errors"
# case = "getlines_missing_file_returns_empty_list"
# subject = "linecache.getlines"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.getlines: getlines of a non-existent file returns [] without raising"""
import linecache

linecache.clearcache()
assert linecache.getlines("/no/such/file.py") == [], "missing file getlines"
print("getlines_missing_file_returns_empty_list OK")
