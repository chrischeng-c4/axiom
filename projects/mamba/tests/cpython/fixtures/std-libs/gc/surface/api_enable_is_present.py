# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "surface"
# case = "api_enable_is_present"
# subject = "gc.enable"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gc.enable: api_enable_is_present (surface)."""
import gc

assert hasattr(gc, "enable")
print("api_enable_is_present OK")
