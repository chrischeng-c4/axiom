# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "autojunk_false_keeps_long_match"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.SequenceMatcher: with autojunk=False a 200-char common run is NOT treated as junk, so ratio of 'a'*200+'b' vs 'a'*200+'c' is > 0.99"""
import difflib

_sm = difflib.SequenceMatcher(
    autojunk=False, a=list("a" * 200 + "b"), b=list("a" * 200 + "c"))
_ratio = _sm.ratio()
assert _ratio > 0.99, f"autojunk=False ratio = {_ratio!r}"  # almost identical
print("autojunk_false_keeps_long_match OK")
