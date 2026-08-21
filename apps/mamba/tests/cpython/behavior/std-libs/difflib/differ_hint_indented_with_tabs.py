# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "differ_hint_indented_with_tabs"
# subject = "difflib.Differ"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.Differ: Differ guide-line indentation expands tabs so the '+' marker lands directly under the changed column"""
import difflib

_diff = list(difflib.Differ().compare(["\t \t \t^"], ["\t \t \t^\n"]))
assert _diff[0] == "- \t \t \t^", f"line0 = {_diff[0]!r}"
assert _diff[1] == "+ \t \t \t^\n", f"line1 = {_diff[1]!r}"
assert _diff[2] == "? \t \t \t +\n", f"line2 = {_diff[2]!r}"
print("differ_hint_indented_with_tabs OK")
