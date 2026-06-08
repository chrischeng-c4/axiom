# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "surface"
# case = "has_token"
# subject = "contextvars"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars: has_token (surface)."""
import contextvars

assert hasattr(contextvars, "Token")
print("has_token OK")
