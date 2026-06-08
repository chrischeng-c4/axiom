# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "surface"
# case = "api_collect_is_present"
# subject = "gc.collect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gc.collect: api_collect_is_present (surface)."""
import gc

assert hasattr(gc, "collect")
print("api_collect_is_present OK")
