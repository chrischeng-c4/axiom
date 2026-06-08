# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "errors"
# case = "short_hex_raises_valueerror"
# subject = "uuid.UUID"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: short_hex_raises_valueerror (errors)."""
import uuid

_raised = False
try:
    uuid.UUID("abc")
except ValueError:
    _raised = True
assert _raised, "short_hex_raises_valueerror: expected ValueError"
print("short_hex_raises_valueerror OK")
