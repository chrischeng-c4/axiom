# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "api_altsep_is_present"
# subject = "os.path.altsep"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.path.altsep: api_altsep_is_present (surface)."""
import os.path

assert hasattr(os.path, "altsep")
print("api_altsep_is_present OK")
