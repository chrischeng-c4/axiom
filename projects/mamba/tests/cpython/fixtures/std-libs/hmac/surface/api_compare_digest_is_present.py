# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "surface"
# case = "api_compare_digest_is_present"
# subject = "hmac.compare_digest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""hmac.compare_digest: api_compare_digest_is_present (surface)."""
import hmac

assert hasattr(hmac, "compare_digest")
print("api_compare_digest_is_present OK")
