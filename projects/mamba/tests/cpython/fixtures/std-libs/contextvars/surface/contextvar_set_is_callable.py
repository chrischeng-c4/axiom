# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "surface"
# case = "contextvar_set_is_callable"
# subject = "contextvars.ContextVar.set"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar.set: contextvar_set_is_callable (surface)."""
import contextvars

assert callable(contextvars.ContextVar.set)
print("contextvar_set_is_callable OK")
