# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "errors"
# case = "bad_string_raises_valueerror"
# subject = "uuid.UUID"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: bad_string_raises_valueerror (errors)."""
import uuid

_raised = False
try:
    uuid.UUID("not_a_uuid")
except ValueError:
    _raised = True
assert _raised, "bad_string_raises_valueerror: expected ValueError"
print("bad_string_raises_valueerror OK")
