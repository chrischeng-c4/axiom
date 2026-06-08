# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "surface"
# case = "api_b2a_base64_is_present"
# subject = "binascii.b2a_base64"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""binascii.b2a_base64: api_b2a_base64_is_present (surface)."""
import binascii

assert hasattr(binascii, "b2a_base64")
print("api_b2a_base64_is_present OK")
