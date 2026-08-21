# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "surface"
# case = "crc32_is_callable"
# subject = "binascii.crc32"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""binascii.crc32: crc32_is_callable (surface)."""
import binascii

assert callable(binascii.crc32)
print("crc32_is_callable OK")
