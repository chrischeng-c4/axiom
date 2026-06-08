# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "api_copymode_is_present"
# subject = "shutil.copymode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""shutil.copymode: api_copymode_is_present (surface)."""
import shutil

assert hasattr(shutil, "copymode")
print("api_copymode_is_present OK")
