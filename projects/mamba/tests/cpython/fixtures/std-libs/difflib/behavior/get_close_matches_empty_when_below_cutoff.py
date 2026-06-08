# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "get_close_matches_empty_when_below_cutoff"
# subject = "difflib.get_close_matches"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.get_close_matches: get_close_matches with a high cutoff and no qualifying word returns []"""
import difflib

_no_match = difflib.get_close_matches(
    "zzzzz", ["apple", "banana", "cherry"], cutoff=0.9)
assert _no_match == [], f"no match = {_no_match!r}"
print("get_close_matches_empty_when_below_cutoff OK")
