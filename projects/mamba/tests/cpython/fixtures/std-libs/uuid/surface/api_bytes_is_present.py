# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "api_bytes_is_present"
# subject = "uuid.bytes_"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""uuid.bytes_: api_bytes_is_present (surface)."""
import uuid

assert hasattr(uuid, "bytes_")
print("api_bytes_is_present OK")
