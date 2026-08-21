# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "check_crc32_not_callable"
# subject = "lzma.CHECK_CRC32"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""lzma.CHECK_CRC32: check_crc32_not_callable (surface)."""
import lzma

assert not callable(lzma.CHECK_CRC32)
print("check_crc32_not_callable OK")
