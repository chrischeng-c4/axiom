# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "surface"
# case = "hmac_update_is_callable"
# subject = "hmac.HMAC.update"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hmac.HMAC.update: hmac_update_is_callable (surface)."""
import hmac

assert callable(hmac.HMAC.update)
print("hmac_update_is_callable OK")
