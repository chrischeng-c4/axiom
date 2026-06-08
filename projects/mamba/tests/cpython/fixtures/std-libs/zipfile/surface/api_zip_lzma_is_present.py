# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "surface"
# case = "api_zip_lzma_is_present"
# subject = "zipfile.ZIP_LZMA"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zipfile.ZIP_LZMA: api_zip_lzma_is_present (surface)."""
import zipfile

assert hasattr(zipfile, "ZIP_LZMA")
print("api_zip_lzma_is_present OK")
