# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "surface"
# case = "api_walk_stack_is_present"
# subject = "traceback.walk_stack"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""traceback.walk_stack: api_walk_stack_is_present (surface)."""
import traceback

assert hasattr(traceback, "walk_stack")
print("api_walk_stack_is_present OK")
