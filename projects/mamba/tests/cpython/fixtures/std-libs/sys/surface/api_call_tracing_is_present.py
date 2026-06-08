# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_call_tracing_is_present"
# subject = "sys.call_tracing"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.call_tracing: api_call_tracing_is_present (surface)."""
import sys

assert hasattr(sys, "call_tracing")
print("api_call_tracing_is_present OK")
