# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "surface"
# case = "encode_is_callable"
# subject = "base64.encode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.encode: encode_is_callable (surface)."""
import base64

assert callable(base64.encode)
print("encode_is_callable OK")
