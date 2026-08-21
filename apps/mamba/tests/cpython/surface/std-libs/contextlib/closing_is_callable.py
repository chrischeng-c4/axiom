# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "surface"
# case = "closing_is_callable"
# subject = "contextlib.closing"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextlib.closing: closing_is_callable (surface)."""
import contextlib

assert callable(contextlib.closing)
print("closing_is_callable OK")
