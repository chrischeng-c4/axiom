# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "differ_added_tab_hint"
# subject = "difflib.Differ"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.Differ: Differ().compare emits a '? ' guide line whose '--'/'+' markers point at the changed (added-tab) columns"""
import difflib

_diff = list(difflib.Differ().compare(["\tI am a buggy"], ["\t\tI am a bug"]))
assert _diff[0] == "- \tI am a buggy", f"line0 = {_diff[0]!r}"
assert _diff[1] == "? \t          --\n", f"line1 = {_diff[1]!r}"
assert _diff[2] == "+ \t\tI am a bug", f"line2 = {_diff[2]!r}"
assert _diff[3] == "? +\n", f"line3 = {_diff[3]!r}"
print("differ_added_tab_hint OK")
