# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "surface"
# case = "zipinfo_is_callable"
# subject = "zipfile.ZipInfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipInfo: zipinfo_is_callable (surface)."""
import zipfile

assert callable(zipfile.ZipInfo)
print("zipinfo_is_callable OK")
