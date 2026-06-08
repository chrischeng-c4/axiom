# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "surface"
# case = "hmac_hexdigest_is_callable"
# subject = "hmac.HMAC.hexdigest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hmac.HMAC.hexdigest: hmac_hexdigest_is_callable (surface)."""
import hmac

assert callable(hmac.HMAC.hexdigest)
print("hmac_hexdigest_is_callable OK")
