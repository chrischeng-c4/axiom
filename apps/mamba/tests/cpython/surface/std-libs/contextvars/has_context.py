# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "surface"
# case = "has_context"
# subject = "contextvars"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars: has_context (surface)."""
import contextvars

assert hasattr(contextvars, "Context")
print("has_context OK")
