# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "surface"
# case = "api_crc32_is_present"
# subject = "binascii.crc32"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""binascii.crc32: api_crc32_is_present (surface)."""
import binascii

assert hasattr(binascii, "crc32")
print("api_crc32_is_present OK")
