# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "surface"
# case = "suppress_is_callable"
# subject = "contextlib.suppress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextlib.suppress: suppress_is_callable (surface)."""
import contextlib

assert callable(contextlib.suppress)
print("suppress_is_callable OK")
