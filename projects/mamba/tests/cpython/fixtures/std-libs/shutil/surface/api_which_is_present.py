# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "api_which_is_present"
# subject = "shutil.which"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""shutil.which: api_which_is_present (surface)."""
import shutil

assert hasattr(shutil, "which")
print("api_which_is_present OK")
