# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "errors"
# case = "non_uuid_namespace_raises_attributeerror"
# subject = "uuid.uuid5"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.uuid5: non_uuid_namespace_raises_attributeerror (errors)."""
import uuid

_raised = False
try:
    uuid.uuid5("not_uuid", "name")
except AttributeError:
    _raised = True
assert _raised, "non_uuid_namespace_raises_attributeerror: expected AttributeError"
print("non_uuid_namespace_raises_attributeerror OK")
