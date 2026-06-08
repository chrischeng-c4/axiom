# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "surface"
# case = "a85decode_is_callable"
# subject = "base64.a85decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.a85decode: a85decode_is_callable (surface)."""
import base64

assert callable(base64.a85decode)
print("a85decode_is_callable OK")
