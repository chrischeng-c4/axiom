# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "surface"
# case = "api_trans_36_is_present"
# subject = "hmac.trans_36"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""hmac.trans_36: api_trans_36_is_present (surface)."""
import hmac

assert hasattr(hmac, "trans_36")
print("api_trans_36_is_present OK")
