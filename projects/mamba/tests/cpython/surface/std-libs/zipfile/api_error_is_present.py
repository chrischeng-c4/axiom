# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "surface"
# case = "api_error_is_present"
# subject = "zipfile.error"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zipfile.error: api_error_is_present (surface)."""
import zipfile

assert hasattr(zipfile, "error")
print("api_error_is_present OK")
