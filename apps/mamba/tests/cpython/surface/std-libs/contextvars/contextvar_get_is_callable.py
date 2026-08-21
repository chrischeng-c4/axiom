# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "surface"
# case = "contextvar_get_is_callable"
# subject = "contextvars.ContextVar.get"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar.get: contextvar_get_is_callable (surface)."""
import contextvars

assert callable(contextvars.ContextVar.get)
print("contextvar_get_is_callable OK")
