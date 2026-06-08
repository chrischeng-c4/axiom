# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "surface"
# case = "api_b32encode_is_present"
# subject = "base64.b32encode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""base64.b32encode: api_b32encode_is_present (surface)."""
import base64

assert hasattr(base64, "b32encode")
print("api_b32encode_is_present OK")
