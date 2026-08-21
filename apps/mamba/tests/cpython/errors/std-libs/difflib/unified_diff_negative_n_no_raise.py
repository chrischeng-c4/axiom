# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "errors"
# case = "unified_diff_negative_n_no_raise"
# subject = "difflib.unified_diff"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.unified_diff: unified_diff(n=-1) does NOT raise; it just yields a clamped (empty-context) diff"""
import difflib

res = list(difflib.unified_diff(["a"], ["b"], n=-1))
assert isinstance(res, list), f"result type = {type(res)!r}"
print("neg_n: lines=", len(res))
print("unified_diff_negative_n_no_raise OK")
