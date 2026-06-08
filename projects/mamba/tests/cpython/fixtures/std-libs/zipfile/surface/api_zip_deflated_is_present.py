# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "surface"
# case = "api_zip_deflated_is_present"
# subject = "zipfile.ZIP_DEFLATED"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zipfile.ZIP_DEFLATED: api_zip_deflated_is_present (surface)."""
import zipfile

assert hasattr(zipfile, "ZIP_DEFLATED")
print("api_zip_deflated_is_present OK")
