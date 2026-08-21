# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "surface"
# case = "singledispatch_is_callable"
# subject = "functools.singledispatch"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""functools.singledispatch: singledispatch_is_callable (surface)."""
import functools

assert callable(functools.singledispatch)
print("singledispatch_is_callable OK")
