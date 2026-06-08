# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "isjunk_classifies_bjunk_set"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.SequenceMatcher: the isjunk predicate populates the bjunk set with the b-sequence elements it flags (just ' ', then ' ' and 'b')"""
import difflib

_sm = difflib.SequenceMatcher(
    isjunk=lambda x: x == " ",
    a="a" * 40 + "b" * 40, b="a" * 44 + "b" * 40 + " " * 20)
assert _sm.bjunk == {" "}, f"bjunk = {_sm.bjunk!r}"
_sm2 = difflib.SequenceMatcher(
    isjunk=lambda x: x in (" ", "b"),
    a="a" * 40 + "b" * 40, b="a" * 44 + "b" * 40 + " " * 20)
assert _sm2.bjunk == {" ", "b"}, f"bjunk multi = {_sm2.bjunk!r}"
print("isjunk_classifies_bjunk_set OK")
