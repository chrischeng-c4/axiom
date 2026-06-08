# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "api_copy_is_present"
# subject = "shutil.copy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""shutil.copy: api_copy_is_present (surface)."""
import shutil

assert hasattr(shutil, "copy")
print("api_copy_is_present OK")
