# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "surface"
# case = "api_digest_is_present"
# subject = "hmac.digest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""hmac.digest: api_digest_is_present (surface)."""
import hmac

assert hasattr(hmac, "digest")
print("api_digest_is_present OK")
