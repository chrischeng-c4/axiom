# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "api_same_file_error_is_present"
# subject = "shutil.SameFileError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""shutil.SameFileError: api_same_file_error_is_present (surface)."""
import shutil

assert hasattr(shutil, "SameFileError")
print("api_same_file_error_is_present OK")
