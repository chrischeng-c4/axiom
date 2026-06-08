# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "api_copy2_is_present"
# subject = "shutil.copy2"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""shutil.copy2: api_copy2_is_present (surface)."""
import shutil

assert hasattr(shutil, "copy2")
print("api_copy2_is_present OK")
