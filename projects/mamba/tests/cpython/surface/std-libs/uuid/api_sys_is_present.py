# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "api_sys_is_present"
# subject = "uuid.sys"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""uuid.sys: api_sys_is_present (surface)."""
import uuid

assert hasattr(uuid, "sys")
print("api_sys_is_present OK")
