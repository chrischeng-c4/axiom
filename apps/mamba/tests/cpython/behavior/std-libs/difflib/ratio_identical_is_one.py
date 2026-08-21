# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "ratio_identical_is_one"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.SequenceMatcher: ratio() of two identical strings is exactly 1.0"""
import difflib

_sm = difflib.SequenceMatcher(None, "abc", "abc")
assert _sm.ratio() == 1.0, f"identical ratio = {_sm.ratio()!r}"
print("ratio_identical_is_one OK")
