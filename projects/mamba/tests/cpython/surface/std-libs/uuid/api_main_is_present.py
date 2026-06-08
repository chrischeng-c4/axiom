# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "api_main_is_present"
# subject = "uuid.main"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""uuid.main: api_main_is_present (surface)."""
import uuid

assert hasattr(uuid, "main")
print("api_main_is_present OK")
