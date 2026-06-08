# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "surface"
# case = "b85decode_is_callable"
# subject = "base64.b85decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.b85decode: b85decode_is_callable (surface)."""
import base64

assert callable(base64.b85decode)
print("b85decode_is_callable OK")
