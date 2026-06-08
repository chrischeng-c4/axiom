# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "errors"
# case = "multiple_constructor_args_raises_typeerror"
# subject = "uuid.UUID"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: multiple_constructor_args_raises_typeerror (errors)."""
import uuid

_raised = False
try:
    uuid.UUID(hex="12345678-1234-1234-1234-123456789012", int=1)
except TypeError:
    _raised = True
assert _raised, "multiple_constructor_args_raises_typeerror: expected TypeError"
print("multiple_constructor_args_raises_typeerror OK")
