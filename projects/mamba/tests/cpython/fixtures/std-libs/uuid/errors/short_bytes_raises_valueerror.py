# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "errors"
# case = "short_bytes_raises_valueerror"
# subject = "uuid.UUID"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: short_bytes_raises_valueerror (errors)."""
import uuid

_raised = False
try:
    uuid.UUID(bytes=bytes(4))
except ValueError:
    _raised = True
assert _raised, "short_bytes_raises_valueerror: expected ValueError"
print("short_bytes_raises_valueerror OK")
