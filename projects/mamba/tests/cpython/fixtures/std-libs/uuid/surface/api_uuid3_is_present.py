# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "api_uuid3_is_present"
# subject = "uuid.uuid3"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""uuid.uuid3: api_uuid3_is_present (surface)."""
import uuid

assert hasattr(uuid, "uuid3")
print("api_uuid3_is_present OK")
