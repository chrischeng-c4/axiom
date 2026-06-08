# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "surface"
# case = "api_extract_stack_is_present"
# subject = "traceback.extract_stack"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""traceback.extract_stack: api_extract_stack_is_present (surface)."""
import traceback

assert hasattr(traceback, "extract_stack")
print("api_extract_stack_is_present OK")
