# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "surface"
# case = "api_stack_summary_is_present"
# subject = "traceback.StackSummary"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""traceback.StackSummary: api_stack_summary_is_present (surface)."""
import traceback

assert hasattr(traceback, "StackSummary")
print("api_stack_summary_is_present OK")
