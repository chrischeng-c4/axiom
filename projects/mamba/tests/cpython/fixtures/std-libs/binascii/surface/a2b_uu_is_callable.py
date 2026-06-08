# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "surface"
# case = "a2b_uu_is_callable"
# subject = "binascii.a2b_uu"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""binascii.a2b_uu: a2b_uu_is_callable (surface)."""
import binascii

assert callable(binascii.a2b_uu)
print("a2b_uu_is_callable OK")
