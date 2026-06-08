# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "surface"
# case = "api_zip_info_is_present"
# subject = "zipfile.ZipInfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zipfile.ZipInfo: api_zip_info_is_present (surface)."""
import zipfile

assert hasattr(zipfile, "ZipInfo")
print("api_zip_info_is_present OK")
