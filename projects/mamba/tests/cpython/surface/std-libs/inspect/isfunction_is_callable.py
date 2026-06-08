# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "isfunction_is_callable"
# subject = "inspect.isfunction"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.isfunction: isfunction_is_callable (surface)."""
import inspect

assert callable(inspect.isfunction)
print("isfunction_is_callable OK")
