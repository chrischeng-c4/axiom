# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "surface"
# case = "api_hmac_is_present"
# subject = "hmac.HMAC"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""hmac.HMAC: api_hmac_is_present (surface)."""
import hmac

assert hasattr(hmac, "HMAC")
print("api_hmac_is_present OK")
