# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "surface"
# case = "partial_is_callable"
# subject = "functools.partial"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""functools.partial: partial_is_callable (surface)."""
import functools

assert callable(functools.partial)
print("partial_is_callable OK")
