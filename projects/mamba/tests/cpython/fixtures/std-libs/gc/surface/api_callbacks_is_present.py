# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "surface"
# case = "api_callbacks_is_present"
# subject = "gc.callbacks"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gc.callbacks: api_callbacks_is_present (surface)."""
import gc

assert hasattr(gc, "callbacks")
print("api_callbacks_is_present OK")
