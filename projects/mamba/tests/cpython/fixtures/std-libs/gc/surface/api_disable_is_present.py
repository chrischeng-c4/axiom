# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "surface"
# case = "api_disable_is_present"
# subject = "gc.disable"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gc.disable: api_disable_is_present (surface)."""
import gc

assert hasattr(gc, "disable")
print("api_disable_is_present OK")
