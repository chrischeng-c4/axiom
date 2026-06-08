# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "surface"
# case = "api_get_referrers_is_present"
# subject = "gc.get_referrers"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gc.get_referrers: api_get_referrers_is_present (surface)."""
import gc

assert hasattr(gc, "get_referrers")
print("api_get_referrers_is_present OK")
