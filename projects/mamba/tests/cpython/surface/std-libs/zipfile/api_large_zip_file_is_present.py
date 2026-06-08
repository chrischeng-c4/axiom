# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "surface"
# case = "api_large_zip_file_is_present"
# subject = "zipfile.LargeZipFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zipfile.LargeZipFile: api_large_zip_file_is_present (surface)."""
import zipfile

assert hasattr(zipfile, "LargeZipFile")
print("api_large_zip_file_is_present OK")
