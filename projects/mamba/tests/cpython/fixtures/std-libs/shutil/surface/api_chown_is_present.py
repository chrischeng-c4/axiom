# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "api_chown_is_present"
# subject = "shutil.chown"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""shutil.chown: api_chown_is_present (surface)."""
import shutil

assert hasattr(shutil, "chown")
print("api_chown_is_present OK")
