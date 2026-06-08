# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "surface"
# case = "crc_hqx_is_callable"
# subject = "binascii.crc_hqx"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""binascii.crc_hqx: crc_hqx_is_callable (surface)."""
import binascii

assert callable(binascii.crc_hqx)
print("crc_hqx_is_callable OK")
