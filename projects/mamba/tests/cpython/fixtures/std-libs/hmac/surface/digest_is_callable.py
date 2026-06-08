# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "surface"
# case = "digest_is_callable"
# subject = "hmac.digest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hmac.digest: digest_is_callable (surface)."""
import hmac

assert callable(hmac.digest)
print("digest_is_callable OK")
