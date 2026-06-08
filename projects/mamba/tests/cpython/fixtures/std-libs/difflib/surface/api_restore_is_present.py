# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "surface"
# case = "api_restore_is_present"
# subject = "difflib.restore"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""difflib.restore: api_restore_is_present (surface)."""
import difflib

assert hasattr(difflib, "restore")
print("api_restore_is_present OK")
