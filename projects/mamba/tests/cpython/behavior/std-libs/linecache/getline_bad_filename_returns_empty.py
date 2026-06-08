# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "getline_bad_filename_returns_empty"
# subject = "linecache.getline"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.getline: getline with an empty or syntactically-invalid filename returns '' without raising"""
import linecache

linecache.clearcache()
assert linecache.getline("", 1) == "", "getline empty name"
assert linecache.getline("!@$)(!@#_1", 1) == "", "getline invalid name"
print("getline_bad_filename_returns_empty OK")
