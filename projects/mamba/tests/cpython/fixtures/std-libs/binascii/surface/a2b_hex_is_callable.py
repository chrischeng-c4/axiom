# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "surface"
# case = "a2b_hex_is_callable"
# subject = "binascii.a2b_hex"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""binascii.a2b_hex: a2b_hex_is_callable (surface)."""
import binascii

assert callable(binascii.a2b_hex)
print("a2b_hex_is_callable OK")
