# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "api_defpath_is_present"
# subject = "os.path.defpath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.path.defpath: api_defpath_is_present (surface)."""
import os.path

assert hasattr(os.path, "defpath")
print("api_defpath_is_present OK")
