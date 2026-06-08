# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "surface"
# case = "encodebytes_is_callable"
# subject = "base64.encodebytes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.encodebytes: encodebytes_is_callable (surface)."""
import base64

assert callable(base64.encodebytes)
print("encodebytes_is_callable OK")
