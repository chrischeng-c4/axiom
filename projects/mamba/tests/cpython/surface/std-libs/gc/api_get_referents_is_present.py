# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "surface"
# case = "api_get_referents_is_present"
# subject = "gc.get_referents"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gc.get_referents: api_get_referents_is_present (surface)."""
import gc

assert hasattr(gc, "get_referents")
print("api_get_referents_is_present OK")
