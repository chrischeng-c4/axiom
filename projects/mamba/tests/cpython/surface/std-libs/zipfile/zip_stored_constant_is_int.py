# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "surface"
# case = "zip_stored_constant_is_int"
# subject = "zipfile.ZIP_STORED"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZIP_STORED: zip_stored_constant_is_int (surface)."""
import zipfile

assert type(zipfile.ZIP_STORED).__name__ == "int"
print("zip_stored_constant_is_int OK")
