# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_trace_is_present"
# subject = "inspect.trace"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.trace: api_trace_is_present (surface)."""
import inspect

assert hasattr(inspect, "trace")
print("api_trace_is_present OK")
