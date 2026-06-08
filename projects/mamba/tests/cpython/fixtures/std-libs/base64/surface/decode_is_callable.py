# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "surface"
# case = "decode_is_callable"
# subject = "base64.decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.decode: decode_is_callable (surface)."""
import base64

assert callable(base64.decode)
print("decode_is_callable OK")
