# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "context_diff_tab_delimited_filedates"
# subject = "difflib.context_diff"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.context_diff: with filedates the context_diff '***'/'---' file headers are tab-separated 'name\\tdate'"""
import difflib

_cd = list(difflib.context_diff(
    ["one"], ["two"], "Original", "Current",
    "2005-01-26 23:30:50", "2010-04-02 10:20:52", lineterm=""))
assert _cd[0] == "*** Original\t2005-01-26 23:30:50", f"from header = {_cd[0]!r}"
assert _cd[1] == "--- Current\t2010-04-02 10:20:52", f"to header = {_cd[1]!r}"
print("context_diff_tab_delimited_filedates OK")
