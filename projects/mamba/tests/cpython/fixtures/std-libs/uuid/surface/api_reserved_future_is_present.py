# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "api_reserved_future_is_present"
# subject = "uuid.RESERVED_FUTURE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""uuid.RESERVED_FUTURE: api_reserved_future_is_present (surface)."""
import uuid

assert hasattr(uuid, "RESERVED_FUTURE")
print("api_reserved_future_is_present OK")
