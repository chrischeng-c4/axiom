# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "errors"
# case = "getline_missing_file_returns_empty"
# subject = "linecache.getline"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.getline: getline of a non-existent file returns '' without raising"""
import linecache

linecache.clearcache()
assert linecache.getline("/no/such/file.py", 1) == "", "missing file getline"
print("getline_missing_file_returns_empty OK")
