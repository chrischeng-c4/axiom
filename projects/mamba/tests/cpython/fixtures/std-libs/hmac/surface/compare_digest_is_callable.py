# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "surface"
# case = "compare_digest_is_callable"
# subject = "hmac.compare_digest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hmac.compare_digest: compare_digest_is_callable (surface)."""
import hmac

assert callable(hmac.compare_digest)
print("compare_digest_is_callable OK")
