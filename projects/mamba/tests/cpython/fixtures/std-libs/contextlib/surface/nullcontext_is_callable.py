# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "surface"
# case = "nullcontext_is_callable"
# subject = "contextlib.nullcontext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextlib.nullcontext: nullcontext_is_callable (surface)."""
import contextlib

assert callable(contextlib.nullcontext)
print("nullcontext_is_callable OK")
