# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "surface"
# case = "reduce_is_callable"
# subject = "functools.reduce"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""functools.reduce: reduce_is_callable (surface)."""
import functools

assert callable(functools.reduce)
print("reduce_is_callable OK")
