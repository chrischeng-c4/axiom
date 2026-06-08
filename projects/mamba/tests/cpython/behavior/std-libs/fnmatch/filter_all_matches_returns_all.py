# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "filter_all_matches_returns_all"
# subject = "fnmatch.filter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fnmatch.filter: filter returns the whole list (a new equal list) when every name matches"""
import fnmatch

_all = ["a.txt", "b.txt", "c.txt"]
_out = fnmatch.filter(_all, "*.txt")
assert _out == _all, f"all match -> {_out!r}"

print("filter_all_matches_returns_all OK")
