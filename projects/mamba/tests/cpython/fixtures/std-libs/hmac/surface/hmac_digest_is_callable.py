# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "surface"
# case = "hmac_digest_is_callable"
# subject = "hmac.HMAC.digest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hmac.HMAC.digest: hmac_digest_is_callable (surface)."""
import hmac

assert callable(hmac.HMAC.digest)
print("hmac_digest_is_callable OK")
