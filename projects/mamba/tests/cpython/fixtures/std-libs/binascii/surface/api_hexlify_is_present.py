# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "surface"
# case = "api_hexlify_is_present"
# subject = "binascii.hexlify"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""binascii.hexlify: api_hexlify_is_present (surface)."""
import binascii

assert hasattr(binascii, "hexlify")
print("api_hexlify_is_present OK")
