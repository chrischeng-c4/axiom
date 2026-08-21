# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "unified_diff_tab_delimited_filedates"
# subject = "difflib.unified_diff"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.unified_diff: with filedates the ---/+++ headers are tab-separated 'name\\tdate'; without filedates there is no trailing tab"""
import difflib

_hdr = list(difflib.unified_diff(
    ["one"], ["two"], "Original", "Current",
    "2005-01-26 23:30:50", "2010-04-02 10:20:52", lineterm=""))
assert _hdr[0] == "--- Original\t2005-01-26 23:30:50", f"from header = {_hdr[0]!r}"
assert _hdr[1] == "+++ Current\t2010-04-02 10:20:52", f"to header = {_hdr[1]!r}"
# Without filedates there is no trailing tab.
_hdr2 = list(difflib.unified_diff(
    ["one"], ["two"], "Original", "Current", lineterm=""))
assert _hdr2[0] == "--- Original", f"bare from header = {_hdr2[0]!r}"
assert _hdr2[1] == "+++ Current", f"bare to header = {_hdr2[1]!r}"
print("unified_diff_tab_delimited_filedates OK")
