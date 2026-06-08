# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "api_expanduser_is_present"
# subject = "os.path.expanduser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.path.expanduser: api_expanduser_is_present (surface)."""
import os.path

assert hasattr(os.path, "expanduser")
print("api_expanduser_is_present OK")
