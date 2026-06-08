# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "surface"
# case = "api_print_exc_is_present"
# subject = "traceback.print_exc"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""traceback.print_exc: api_print_exc_is_present (surface)."""
import traceback

assert hasattr(traceback, "print_exc")
print("api_print_exc_is_present OK")
