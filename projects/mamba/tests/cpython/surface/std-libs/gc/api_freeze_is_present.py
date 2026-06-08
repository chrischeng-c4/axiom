# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "surface"
# case = "api_freeze_is_present"
# subject = "gc.freeze"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gc.freeze: api_freeze_is_present (surface)."""
import gc

assert hasattr(gc, "freeze")
print("api_freeze_is_present OK")
