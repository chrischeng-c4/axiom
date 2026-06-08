# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "api_copyfile_is_present"
# subject = "shutil.copyfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""shutil.copyfile: api_copyfile_is_present (surface)."""
import shutil

assert hasattr(shutil, "copyfile")
print("api_copyfile_is_present OK")
