# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "surface"
# case = "api_unhexlify_is_present"
# subject = "binascii.unhexlify"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""binascii.unhexlify: api_unhexlify_is_present (surface)."""
import binascii

assert hasattr(binascii, "unhexlify")
print("api_unhexlify_is_present OK")
