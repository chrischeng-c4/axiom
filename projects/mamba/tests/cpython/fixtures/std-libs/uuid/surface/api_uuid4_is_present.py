# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "api_uuid4_is_present"
# subject = "uuid.uuid4"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""uuid.uuid4: api_uuid4_is_present (surface)."""
import uuid

assert hasattr(uuid, "uuid4")
print("api_uuid4_is_present OK")
