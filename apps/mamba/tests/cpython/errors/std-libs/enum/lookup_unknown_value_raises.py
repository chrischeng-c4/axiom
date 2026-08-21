# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "errors"
# case = "lookup_unknown_value_raises"
# subject = "enum.Enum"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: lookup_unknown_value_raises (errors)."""
import enum

_raised = False
try:
    enum.Enum('Color', {'RED': 1})(99)
except ValueError:
    _raised = True
assert _raised, "lookup_unknown_value_raises: expected ValueError"
print("lookup_unknown_value_raises OK")
