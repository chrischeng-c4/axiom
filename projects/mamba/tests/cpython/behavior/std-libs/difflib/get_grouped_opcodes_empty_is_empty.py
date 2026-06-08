# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "get_grouped_opcodes_empty_is_empty"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.SequenceMatcher: get_grouped_opcodes() over two empty sequences yields nothing (next() raises StopIteration)"""
import difflib

_grp = difflib.SequenceMatcher(None, [], []).get_grouped_opcodes()
_raised = False
try:
    next(_grp)
except StopIteration:
    _raised = True
assert _raised, "expected empty grouped opcodes"
print("get_grouped_opcodes_empty_is_empty OK")
