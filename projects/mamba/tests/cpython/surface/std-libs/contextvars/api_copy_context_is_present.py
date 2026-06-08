# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "surface"
# case = "api_copy_context_is_present"
# subject = "contextvars.copy_context"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""contextvars.copy_context: api_copy_context_is_present (surface)."""
import contextvars

assert hasattr(contextvars, "copy_context")
print("api_copy_context_is_present OK")
