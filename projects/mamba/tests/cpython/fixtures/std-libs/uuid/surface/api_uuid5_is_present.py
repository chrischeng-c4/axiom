# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "api_uuid5_is_present"
# subject = "uuid.uuid5"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""uuid.uuid5: api_uuid5_is_present (surface)."""
import uuid

assert hasattr(uuid, "uuid5")
print("api_uuid5_is_present OK")
