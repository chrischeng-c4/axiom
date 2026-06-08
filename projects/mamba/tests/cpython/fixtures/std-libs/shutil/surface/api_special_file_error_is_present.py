# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "api_special_file_error_is_present"
# subject = "shutil.SpecialFileError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""shutil.SpecialFileError: api_special_file_error_is_present (surface)."""
import shutil

assert hasattr(shutil, "SpecialFileError")
print("api_special_file_error_is_present OK")
