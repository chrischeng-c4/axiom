# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "api_copystat_is_present"
# subject = "shutil.copystat"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""shutil.copystat: api_copystat_is_present (surface)."""
import shutil

assert hasattr(shutil, "copystat")
print("api_copystat_is_present OK")
