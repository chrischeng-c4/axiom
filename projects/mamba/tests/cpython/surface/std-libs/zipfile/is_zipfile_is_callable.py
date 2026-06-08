# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "surface"
# case = "is_zipfile_is_callable"
# subject = "zipfile.is_zipfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.is_zipfile: is_zipfile_is_callable (surface)."""
import zipfile

assert callable(zipfile.is_zipfile)
print("is_zipfile_is_callable OK")
