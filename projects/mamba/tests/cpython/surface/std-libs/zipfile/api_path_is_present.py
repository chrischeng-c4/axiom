# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "surface"
# case = "api_path_is_present"
# subject = "zipfile.Path"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zipfile.Path: api_path_is_present (surface)."""
import zipfile

assert hasattr(zipfile, "Path")
print("api_path_is_present OK")
