# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "surface"
# case = "api_isenabled_is_present"
# subject = "gc.isenabled"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gc.isenabled: api_isenabled_is_present (surface)."""
import gc

assert hasattr(gc, "isenabled")
print("api_isenabled_is_present OK")
