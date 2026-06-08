# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "surface"
# case = "zipfile_is_callable"
# subject = "zipfile.ZipFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: zipfile_is_callable (surface)."""
import zipfile

assert callable(zipfile.ZipFile)
print("zipfile_is_callable OK")
