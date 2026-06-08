# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "errors"
# case = "member_reassign_raises"
# subject = "enum.Enum"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: member_reassign_raises (errors)."""
import enum

_raised = False
try:
    setattr(enum.Enum('Color', {'RED': 1}), 'RED', 99)
except AttributeError:
    _raised = True
assert _raised, "member_reassign_raises: expected AttributeError"
print("member_reassign_raises OK")
