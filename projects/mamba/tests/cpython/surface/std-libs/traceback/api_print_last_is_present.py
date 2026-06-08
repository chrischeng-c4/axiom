# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "surface"
# case = "api_print_last_is_present"
# subject = "traceback.print_last"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""traceback.print_last: api_print_last_is_present (surface)."""
import traceback

assert hasattr(traceback, "print_last")
print("api_print_last_is_present OK")
