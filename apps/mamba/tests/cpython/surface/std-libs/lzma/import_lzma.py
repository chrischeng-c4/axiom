# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "import_lzma"
# subject = "lzma"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""lzma: import_lzma (surface)."""
import lzma

assert hasattr(lzma, "compress")
print("import_lzma OK")
