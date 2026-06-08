# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "surface"
# case = "api_format_exception_is_present"
# subject = "traceback.format_exception"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""traceback.format_exception: api_format_exception_is_present (surface)."""
import traceback

assert hasattr(traceback, "format_exception")
print("api_format_exception_is_present OK")
