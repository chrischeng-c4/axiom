# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "api_uuid1_is_present"
# subject = "uuid.uuid1"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""uuid.uuid1: api_uuid1_is_present (surface)."""
import uuid

assert hasattr(uuid, "uuid1")
print("api_uuid1_is_present OK")
