# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "api_uuid_is_present"
# subject = "uuid.UUID"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""uuid.UUID: api_uuid_is_present (surface)."""
import uuid

assert hasattr(uuid, "UUID")
print("api_uuid_is_present OK")
