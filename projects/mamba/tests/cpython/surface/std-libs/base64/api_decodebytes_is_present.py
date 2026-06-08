# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "surface"
# case = "api_decodebytes_is_present"
# subject = "base64.decodebytes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""base64.decodebytes: api_decodebytes_is_present (surface)."""
import base64

assert hasattr(base64, "decodebytes")
print("api_decodebytes_is_present OK")
