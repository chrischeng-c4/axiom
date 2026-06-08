# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "surface"
# case = "decodebytes_is_callable"
# subject = "base64.decodebytes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.decodebytes: decodebytes_is_callable (surface)."""
import base64

assert callable(base64.decodebytes)
print("decodebytes_is_callable OK")
