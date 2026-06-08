# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "api_os_is_present"
# subject = "uuid.os"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""uuid.os: api_os_is_present (surface)."""
import uuid

assert hasattr(uuid, "os")
print("api_os_is_present OK")
