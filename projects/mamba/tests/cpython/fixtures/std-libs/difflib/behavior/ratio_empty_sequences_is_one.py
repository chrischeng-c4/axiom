# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "ratio_empty_sequences_is_one"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.SequenceMatcher: ratio/quick_ratio/real_quick_ratio of two empty sequences are all 1.0 (vacuously identical)"""
import difflib

_sm = difflib.SequenceMatcher(None, [], [])
assert _sm.ratio() == 1.0, f"empty ratio = {_sm.ratio()!r}"
assert _sm.quick_ratio() == 1.0, f"empty quick_ratio = {_sm.quick_ratio()!r}"
assert _sm.real_quick_ratio() == 1.0, f"empty real_quick_ratio = {_sm.real_quick_ratio()!r}"
print("ratio_empty_sequences_is_one OK")
