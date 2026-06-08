# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "unified_diff_hunk_header"
# subject = "difflib.unified_diff"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.unified_diff: unified_diff emits an '@@ -1,3 +1,3 @@' hunk header for a 3-line file with one replaced line"""
import difflib

_a = "a\nb\nc\n".splitlines(keepends=True)
_b = "a\nX\nc\n".splitlines(keepends=True)
_ud = list(difflib.unified_diff(_a, _b, lineterm=""))
assert "@@ -1,3 +1,3 @@" in _ud, f"unified hunk header = {_ud!r}"
print("unified_diff_hunk_header OK")
