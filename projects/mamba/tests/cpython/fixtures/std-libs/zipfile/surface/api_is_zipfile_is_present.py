# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "surface"
# case = "api_is_zipfile_is_present"
# subject = "zipfile.is_zipfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zipfile.is_zipfile: api_is_zipfile_is_present (surface)."""
import zipfile

assert hasattr(zipfile, "is_zipfile")
print("api_is_zipfile_is_present OK")
