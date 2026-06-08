# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "api_enum_is_present"
# subject = "uuid.Enum"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""uuid.Enum: api_enum_is_present (surface)."""
import uuid

assert hasattr(uuid, "Enum")
print("api_enum_is_present OK")
