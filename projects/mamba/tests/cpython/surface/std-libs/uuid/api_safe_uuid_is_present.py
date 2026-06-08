# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "api_safe_uuid_is_present"
# subject = "uuid.SafeUUID"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""uuid.SafeUUID: api_safe_uuid_is_present (surface)."""
import uuid

assert hasattr(uuid, "SafeUUID")
print("api_safe_uuid_is_present OK")
