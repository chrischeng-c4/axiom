# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "surface"
# case = "api_error_is_present"
# subject = "binascii.Error"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""binascii.Error: api_error_is_present (surface)."""
import binascii

assert hasattr(binascii, "Error")
print("api_error_is_present OK")
