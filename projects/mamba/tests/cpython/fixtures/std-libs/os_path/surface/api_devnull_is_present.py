# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "api_devnull_is_present"
# subject = "os.path.devnull"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.path.devnull: api_devnull_is_present (surface)."""
import os.path

assert hasattr(os.path, "devnull")
print("api_devnull_is_present OK")
