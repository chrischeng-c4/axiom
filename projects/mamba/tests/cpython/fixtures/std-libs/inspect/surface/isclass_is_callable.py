# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "isclass_is_callable"
# subject = "inspect.isclass"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.isclass: isclass_is_callable (surface)."""
import inspect

assert callable(inspect.isclass)
print("isclass_is_callable OK")
