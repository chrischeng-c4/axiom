# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "surface"
# case = "b32hexencode_is_callable"
# subject = "base64.b32hexencode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.b32hexencode: b32hexencode_is_callable (surface)."""
import base64

assert callable(base64.b32hexencode)
print("b32hexencode_is_callable OK")
