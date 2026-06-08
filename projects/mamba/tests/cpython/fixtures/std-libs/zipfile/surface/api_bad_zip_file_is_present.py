# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "surface"
# case = "api_bad_zip_file_is_present"
# subject = "zipfile.BadZipFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zipfile.BadZipFile: api_bad_zip_file_is_present (surface)."""
import zipfile

assert hasattr(zipfile, "BadZipFile")
print("api_bad_zip_file_is_present OK")
