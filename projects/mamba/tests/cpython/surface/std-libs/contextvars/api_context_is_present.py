# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "surface"
# case = "api_context_is_present"
# subject = "contextvars.Context"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""contextvars.Context: api_context_is_present (surface)."""
import contextvars

assert hasattr(contextvars, "Context")
print("api_context_is_present OK")
