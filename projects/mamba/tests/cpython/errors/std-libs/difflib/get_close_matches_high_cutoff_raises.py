# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "errors"
# case = "get_close_matches_high_cutoff_raises"
# subject = "difflib.get_close_matches"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.get_close_matches: get_close_matches_high_cutoff_raises (errors)."""
import difflib

_raised = False
try:
    difflib.get_close_matches("a", ["a"], cutoff=1.5)
except ValueError:
    _raised = True
assert _raised, "get_close_matches_high_cutoff_raises: expected ValueError"
print("get_close_matches_high_cutoff_raises OK")
