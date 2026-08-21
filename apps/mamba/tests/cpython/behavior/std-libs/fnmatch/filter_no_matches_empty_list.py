# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "filter_no_matches_empty_list"
# subject = "fnmatch.filter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fnmatch.filter: filter returns [] when nothing matches the pattern"""
import fnmatch

assert fnmatch.filter(["a.py", "b.rs"], "*.txt") == [], "no matches yields empty list"

print("filter_no_matches_empty_list OK")
