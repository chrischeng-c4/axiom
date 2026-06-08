# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "trace"
# dimension = "surface"
# case = "api_trace_is_present"
# subject = "trace.Trace"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""trace.Trace: api_trace_is_present (surface)."""
import trace

assert hasattr(trace, "Trace")
print("api_trace_is_present OK")
