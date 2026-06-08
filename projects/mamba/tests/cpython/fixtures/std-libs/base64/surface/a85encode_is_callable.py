# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "surface"
# case = "a85encode_is_callable"
# subject = "base64.a85encode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.a85encode: a85encode_is_callable (surface)."""
import base64

assert callable(base64.a85encode)
print("a85encode_is_callable OK")
