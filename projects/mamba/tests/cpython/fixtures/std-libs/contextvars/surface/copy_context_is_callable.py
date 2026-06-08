# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "surface"
# case = "copy_context_is_callable"
# subject = "contextvars.copy_context"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.copy_context: copy_context_is_callable (surface)."""
import contextvars

assert callable(contextvars.copy_context)
print("copy_context_is_callable OK")
