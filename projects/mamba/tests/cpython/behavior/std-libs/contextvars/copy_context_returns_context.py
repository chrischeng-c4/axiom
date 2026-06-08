# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "copy_context_returns_context"
# subject = "contextvars.copy_context"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.copy_context: copy_context() returns a contextvars.Context instance"""
import contextvars

ctx = contextvars.copy_context()
assert isinstance(ctx, contextvars.Context), f"copy_context() type = {type(ctx)!r}"
print("copy_context_returns_context OK")
