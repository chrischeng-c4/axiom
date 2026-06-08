# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "surface"
# case = "context_run_is_callable"
# subject = "contextvars.Context.run"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.Context.run: context_run_is_callable (surface)."""
import contextvars

assert callable(contextvars.Context.run)
print("context_run_is_callable OK")
