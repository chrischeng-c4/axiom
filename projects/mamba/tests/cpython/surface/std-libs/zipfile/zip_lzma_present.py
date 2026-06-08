# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "surface"
# case = "zip_lzma_present"
# subject = "zipfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile: zip_lzma_present (surface)."""
import zipfile

assert hasattr(zipfile, "ZIP_LZMA")
print("zip_lzma_present OK")
