# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "surface"
# case = "b2a_uu_is_callable"
# subject = "binascii.b2a_uu"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""binascii.b2a_uu: b2a_uu_is_callable (surface)."""
import binascii

assert callable(binascii.b2a_uu)
print("b2a_uu_is_callable OK")
