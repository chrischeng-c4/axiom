# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "surface"
# case = "api_a2b_base64_is_present"
# subject = "binascii.a2b_base64"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""binascii.a2b_base64: api_a2b_base64_is_present (surface)."""
import binascii

assert hasattr(binascii, "a2b_base64")
print("api_a2b_base64_is_present OK")
