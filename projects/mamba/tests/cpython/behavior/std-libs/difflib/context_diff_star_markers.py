# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "context_diff_star_markers"
# subject = "difflib.context_diff"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.context_diff: context_diff emits '*** 1,3 ****' (from-range) and '--- 1,3 ----' (to-range) markers for a 3-line file with one change"""
import difflib

_a = "a\nb\nc\n".splitlines(keepends=True)
_b = "a\nX\nc\n".splitlines(keepends=True)
_cd = list(difflib.context_diff(_a, _b, lineterm=""))
assert "*** 1,3 ****" in _cd, f"context from-range = {_cd!r}"
assert "--- 1,3 ----" in _cd, f"context to-range = {_cd!r}"
print("context_diff_star_markers OK")
