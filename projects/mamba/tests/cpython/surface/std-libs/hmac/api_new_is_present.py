# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "surface"
# case = "api_new_is_present"
# subject = "hmac.new"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""hmac.new: api_new_is_present (surface)."""
import hmac

assert hasattr(hmac, "new")
print("api_new_is_present OK")
