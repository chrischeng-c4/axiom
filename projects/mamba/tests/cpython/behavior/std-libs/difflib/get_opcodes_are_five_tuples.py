# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "get_opcodes_are_five_tuples"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.SequenceMatcher: get_opcodes returns 5-tuples whose tags are drawn from {equal, insert, delete, replace}"""
import difflib

_sm = difflib.SequenceMatcher(None, "hello", "helo")
_ops = _sm.get_opcodes()
assert isinstance(_ops, list), f"opcodes type = {type(_ops)!r}"
assert all(len(op) == 5 for op in _ops), "opcodes are 5-tuples"
_tags = {op[0] for op in _ops}
assert _tags <= {"equal", "insert", "delete", "replace"}, f"valid tags = {_tags!r}"
print("get_opcodes_are_five_tuples OK")
