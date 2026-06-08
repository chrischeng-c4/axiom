# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "surface"
# case = "api_standard_b64encode_is_present"
# subject = "base64.standard_b64encode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""base64.standard_b64encode: api_standard_b64encode_is_present (surface)."""
import base64

assert hasattr(base64, "standard_b64encode")
print("api_standard_b64encode_is_present OK")
