# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "repr_is_reconstructable"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: repr(UUID('12345678-1234-5678-1234-567812345678')) is the reconstructable "UUID('12345678-1234-5678-1234-567812345678')" """
import uuid

u = uuid.UUID("12345678-1234-5678-1234-567812345678")
assert repr(u) == "UUID('12345678-1234-5678-1234-567812345678')", f"repr = {repr(u)!r}"
print("repr_is_reconstructable OK")
