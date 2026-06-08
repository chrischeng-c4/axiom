# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "surface"
# case = "unhexlify_is_callable"
# subject = "binascii.unhexlify"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""binascii.unhexlify: unhexlify_is_callable (surface)."""
import binascii

assert callable(binascii.unhexlify)
print("unhexlify_is_callable OK")
