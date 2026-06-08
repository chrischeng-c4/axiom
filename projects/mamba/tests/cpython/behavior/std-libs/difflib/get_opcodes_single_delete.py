# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "get_opcodes_single_delete"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.SequenceMatcher: a single middle deletion yields exactly [equal 0,40,0,40; delete 40,41,40,40; equal 41,81,40,80] and ratio ~= 0.994"""
import difflib

_sm = difflib.SequenceMatcher(None, "a" * 40 + "c" + "b" * 40, "a" * 40 + "b" * 40)
assert round(_sm.ratio(), 3) == 0.994, f"ratio = {_sm.ratio()!r}"
assert list(_sm.get_opcodes()) == [
    ("equal", 0, 40, 0, 40),
    ("delete", 40, 41, 40, 40),
    ("equal", 41, 81, 40, 80),
], f"opcodes = {list(_sm.get_opcodes())!r}"
print("get_opcodes_single_delete OK")
