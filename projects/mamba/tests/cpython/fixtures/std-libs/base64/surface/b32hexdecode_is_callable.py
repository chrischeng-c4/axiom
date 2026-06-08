# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "surface"
# case = "b32hexdecode_is_callable"
# subject = "base64.b32hexdecode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.b32hexdecode: b32hexdecode_is_callable (surface)."""
import base64

assert callable(base64.b32hexdecode)
print("b32hexdecode_is_callable OK")
