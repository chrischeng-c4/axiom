# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "get_opcodes_single_insert"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.SequenceMatcher: a single leading insertion ('b'*100 vs 'a'+'b'*100) yields exactly [insert 0,0,0,1; equal 0,100,1,101] and ratio ~= 0.995"""
import difflib

_sm = difflib.SequenceMatcher(None, "b" * 100, "a" + "b" * 100)
assert round(_sm.ratio(), 3) == 0.995, f"ratio = {_sm.ratio()!r}"
assert list(_sm.get_opcodes()) == [
    ("insert", 0, 0, 0, 1),
    ("equal", 0, 100, 1, 101),
], f"opcodes = {list(_sm.get_opcodes())!r}"
assert _sm.bpopular == set(), f"bpopular = {_sm.bpopular!r}"
print("get_opcodes_single_insert OK")
