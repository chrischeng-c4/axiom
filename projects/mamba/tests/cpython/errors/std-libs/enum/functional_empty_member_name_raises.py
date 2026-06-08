# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "errors"
# case = "functional_empty_member_name_raises"
# subject = "enum.Enum"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: functional_empty_member_name_raises (errors)."""
import enum

_raised = False
try:
    enum.Enum('Empty', ('', 'B', 'C'))
except ValueError:
    _raised = True
assert _raised, "functional_empty_member_name_raises: expected ValueError"
print("functional_empty_member_name_raises OK")
