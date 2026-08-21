# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "surface"
# case = "contextvar_reset_is_callable"
# subject = "contextvars.ContextVar.reset"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar.reset: contextvar_reset_is_callable (surface)."""
import contextvars

assert callable(contextvars.ContextVar.reset)
print("contextvar_reset_is_callable OK")
