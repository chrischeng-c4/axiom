# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "surface"
# case = "singledispatchmethod_is_callable"
# subject = "functools.singledispatchmethod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""functools.singledispatchmethod: singledispatchmethod_is_callable (surface)."""
import functools

assert callable(functools.singledispatchmethod)
print("singledispatchmethod_is_callable OK")
