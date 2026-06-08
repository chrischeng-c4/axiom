# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "errors"
# case = "set_attr_on_immutable_uuid_raises"
# subject = "uuid.UUID"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: set_attr_on_immutable_uuid_raises (errors)."""
import uuid

_raised = False
try:
    setattr(uuid.uuid4(), "hex", "x")
except TypeError:
    _raised = True
assert _raised, "set_attr_on_immutable_uuid_raises: expected TypeError"
print("set_attr_on_immutable_uuid_raises OK")
