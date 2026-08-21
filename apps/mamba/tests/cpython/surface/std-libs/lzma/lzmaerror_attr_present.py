# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "lzmaerror_attr_present"
# subject = "lzma"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""lzma: lzmaerror_attr_present (surface)."""
import lzma

assert hasattr(lzma, "LZMAError")
print("lzmaerror_attr_present OK")
