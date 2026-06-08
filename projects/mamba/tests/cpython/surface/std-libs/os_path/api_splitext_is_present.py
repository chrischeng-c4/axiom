# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "api_splitext_is_present"
# subject = "os.path.splitext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.path.splitext: api_splitext_is_present (surface)."""
import os.path

assert hasattr(os.path, "splitext")
print("api_splitext_is_present OK")
