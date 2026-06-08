# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "surface"
# case = "zip_bzip2_present"
# subject = "zipfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile: zip_bzip2_present (surface)."""
import zipfile

assert hasattr(zipfile, "ZIP_BZIP2")
print("zip_bzip2_present OK")
