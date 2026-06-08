# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "surface"
# case = "import_zlib"
# subject = "zlib"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib: import_zlib (surface)."""
import zlib

assert hasattr(zlib, "compress")
print("import_zlib OK")
