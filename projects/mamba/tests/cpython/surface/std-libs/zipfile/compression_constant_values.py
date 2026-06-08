# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "surface"
# case = "compression_constant_values"
# subject = "zipfile.ZIP_STORED"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZIP_STORED: ZIP_STORED == 0 and ZIP_DEFLATED == 8 (the canonical CPython 3.12 method codes)"""
import zipfile

assert zipfile.ZIP_STORED == 0, f"ZIP_STORED = {zipfile.ZIP_STORED!r}"
assert zipfile.ZIP_DEFLATED == 8, f"ZIP_DEFLATED = {zipfile.ZIP_DEFLATED!r}"

print("compression_constant_values OK")
