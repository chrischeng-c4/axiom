# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "api_int_is_present"
# subject = "uuid.int_"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""uuid.int_: api_int_is_present (surface)."""
import uuid

assert hasattr(uuid, "int_")
print("api_int_is_present OK")
