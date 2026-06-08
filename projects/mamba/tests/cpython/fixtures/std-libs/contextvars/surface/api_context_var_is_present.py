# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "surface"
# case = "api_context_var_is_present"
# subject = "contextvars.ContextVar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""contextvars.ContextVar: api_context_var_is_present (surface)."""
import contextvars

assert hasattr(contextvars, "ContextVar")
print("api_context_var_is_present OK")
