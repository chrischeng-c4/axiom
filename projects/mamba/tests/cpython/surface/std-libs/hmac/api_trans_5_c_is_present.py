# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "surface"
# case = "api_trans_5_c_is_present"
# subject = "hmac.trans_5C"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""hmac.trans_5C: api_trans_5_c_is_present (surface)."""
import hmac

assert hasattr(hmac, "trans_5C")
print("api_trans_5_c_is_present OK")
