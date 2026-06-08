# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "api_error_is_present"
# subject = "shutil.Error"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""shutil.Error: api_error_is_present (surface)."""
import shutil

assert hasattr(shutil, "Error")
print("api_error_is_present OK")
