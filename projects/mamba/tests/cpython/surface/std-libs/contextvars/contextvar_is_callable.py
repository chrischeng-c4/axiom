# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "surface"
# case = "contextvar_is_callable"
# subject = "contextvars.ContextVar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: contextvar_is_callable (surface)."""
import contextvars

assert callable(contextvars.ContextVar)
print("contextvar_is_callable OK")
