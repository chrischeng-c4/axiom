# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "surface"
# case = "api_urlsafe_b64decode_is_present"
# subject = "base64.urlsafe_b64decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""base64.urlsafe_b64decode: api_urlsafe_b64decode_is_present (surface)."""
import base64

assert hasattr(base64, "urlsafe_b64decode")
print("api_urlsafe_b64decode_is_present OK")
