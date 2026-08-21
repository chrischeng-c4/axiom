# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "errors"
# case = "getline_out_of_range_returns_empty"
# subject = "linecache.getline"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.getline: getline with a line number past EOF returns '' without raising"""
import linecache

linecache.clearcache()
# This very file exists but has far fewer than 999999 lines.
assert linecache.getline(__file__, 999999) == "", "out-of-range getline"
print("getline_out_of_range_returns_empty OK")
