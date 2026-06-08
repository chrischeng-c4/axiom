# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "surface"
# case = "api_incomplete_is_present"
# subject = "binascii.Incomplete"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""binascii.Incomplete: api_incomplete_is_present (surface)."""
import binascii

assert hasattr(binascii, "Incomplete")
print("api_incomplete_is_present OK")
