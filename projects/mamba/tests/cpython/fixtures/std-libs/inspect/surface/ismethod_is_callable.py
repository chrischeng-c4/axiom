# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "ismethod_is_callable"
# subject = "inspect.ismethod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.ismethod: ismethod_is_callable (surface)."""
import inspect

assert callable(inspect.ismethod)
print("ismethod_is_callable OK")
