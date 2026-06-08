# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "surface"
# case = "partialmethod_is_callable"
# subject = "functools.partialmethod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""functools.partialmethod: partialmethod_is_callable (surface)."""
import functools

assert callable(functools.partialmethod)
print("partialmethod_is_callable OK")
