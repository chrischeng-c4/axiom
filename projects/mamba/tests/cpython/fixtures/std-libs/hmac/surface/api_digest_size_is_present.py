# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "surface"
# case = "api_digest_size_is_present"
# subject = "hmac.digest_size"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""hmac.digest_size: api_digest_size_is_present (surface)."""
import hmac

assert hasattr(hmac, "digest_size")
print("api_digest_size_is_present OK")
