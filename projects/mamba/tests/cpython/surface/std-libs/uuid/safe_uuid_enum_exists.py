# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "safe_uuid_enum_exists"
# subject = "uuid.SafeUUID"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""uuid.SafeUUID: safe_uuid_enum_exists (surface)."""
import uuid

assert hasattr(uuid.SafeUUID, "safe")
print("safe_uuid_enum_exists OK")
