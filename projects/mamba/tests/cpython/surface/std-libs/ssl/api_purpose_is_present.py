# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_purpose_is_present"
# subject = "ssl.Purpose"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.Purpose: api_purpose_is_present (surface)."""
import ssl

assert hasattr(ssl, "Purpose")
print("api_purpose_is_present OK")
