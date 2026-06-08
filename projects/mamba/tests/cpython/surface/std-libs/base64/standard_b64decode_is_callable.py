# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "surface"
# case = "standard_b64decode_is_callable"
# subject = "base64.standard_b64decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.standard_b64decode: standard_b64decode_is_callable (surface)."""
import base64

assert callable(base64.standard_b64decode)
print("standard_b64decode_is_callable OK")
