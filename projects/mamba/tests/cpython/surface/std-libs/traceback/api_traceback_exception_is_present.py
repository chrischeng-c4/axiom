# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "surface"
# case = "api_traceback_exception_is_present"
# subject = "traceback.TracebackException"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""traceback.TracebackException: api_traceback_exception_is_present (surface)."""
import traceback

assert hasattr(traceback, "TracebackException")
print("api_traceback_exception_is_present OK")
