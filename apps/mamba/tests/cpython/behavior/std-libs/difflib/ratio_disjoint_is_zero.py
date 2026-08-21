# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "ratio_disjoint_is_zero"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.SequenceMatcher: ratio() of two strings with no common characters is exactly 0.0"""
import difflib

_sm = difflib.SequenceMatcher(None, "abc", "xyz")
assert _sm.ratio() == 0.0, f"different ratio = {_sm.ratio()!r}"
print("ratio_disjoint_is_zero OK")
