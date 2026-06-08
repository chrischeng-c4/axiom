# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "surface"
# case = "hmac_copy_is_callable"
# subject = "hmac.HMAC.copy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hmac.HMAC.copy: hmac_copy_is_callable (surface)."""
import hmac

assert callable(hmac.HMAC.copy)
print("hmac_copy_is_callable OK")
