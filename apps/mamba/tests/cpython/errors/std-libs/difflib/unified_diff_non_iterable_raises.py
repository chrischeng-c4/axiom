# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "errors"
# case = "unified_diff_non_iterable_raises"
# subject = "difflib.unified_diff"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.unified_diff: unified_diff_non_iterable_raises (errors)."""
import difflib

_raised = False
try:
    list(difflib.unified_diff(123, ["a"]))
except TypeError:
    _raised = True
assert _raised, "unified_diff_non_iterable_raises: expected TypeError"
print("unified_diff_non_iterable_raises OK")
