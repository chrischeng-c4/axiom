# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "filter_keeps_matching_in_order"
# subject = "fnmatch.filter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.filter: filter returns the matching names in input order; mixed list filtered by '*.txt' keeps only the .txt names in their original positions"""
import fnmatch

_names = ["b.txt", "a.py", "c.txt", "d.txt", "e.py"]
_filtered = fnmatch.filter(_names, "*.txt")
assert _filtered == ["b.txt", "c.txt", "d.txt"], f"filter order = {_filtered!r}"

print("filter_keeps_matching_in_order OK")
