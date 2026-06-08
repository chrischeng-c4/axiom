# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "api_splitroot_is_present"
# subject = "os.path.splitroot"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.path.splitroot: api_splitroot_is_present (surface)."""
import os.path

assert hasattr(os.path, "splitroot")
print("api_splitroot_is_present OK")
