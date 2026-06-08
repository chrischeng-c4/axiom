# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "surface"
# case = "hexlify_is_callable"
# subject = "binascii.hexlify"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""binascii.hexlify: hexlify_is_callable (surface)."""
import binascii

assert callable(binascii.hexlify)
print("hexlify_is_callable OK")
