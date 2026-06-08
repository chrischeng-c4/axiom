# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "api_check_crc32_is_present"
# subject = "lzma.CHECK_CRC32"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""lzma.CHECK_CRC32: api_check_crc32_is_present (surface)."""
import lzma

assert hasattr(lzma, "CHECK_CRC32")
print("api_check_crc32_is_present OK")
