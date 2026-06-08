# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "surface"
# case = "api_is_finalized_is_present"
# subject = "gc.is_finalized"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gc.is_finalized: api_is_finalized_is_present (surface)."""
import gc

assert hasattr(gc, "is_finalized")
print("api_is_finalized_is_present OK")
