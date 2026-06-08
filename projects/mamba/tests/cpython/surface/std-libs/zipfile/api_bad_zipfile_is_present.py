# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "surface"
# case = "api_bad_zipfile_is_present"
# subject = "zipfile.BadZipfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zipfile.BadZipfile: api_bad_zipfile_is_present (surface)."""
import zipfile

assert hasattr(zipfile, "BadZipfile")
print("api_bad_zipfile_is_present OK")
