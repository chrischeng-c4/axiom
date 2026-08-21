# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "surface"
# case = "zip_deflated_constant_is_int"
# subject = "zipfile.ZIP_DEFLATED"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZIP_DEFLATED: zip_deflated_constant_is_int (surface)."""
import zipfile

assert type(zipfile.ZIP_DEFLATED).__name__ == "int"
print("zip_deflated_constant_is_int OK")
