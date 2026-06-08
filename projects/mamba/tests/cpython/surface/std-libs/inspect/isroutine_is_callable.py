# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "isroutine_is_callable"
# subject = "inspect.isroutine"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.isroutine: isroutine_is_callable (surface)."""
import inspect

assert callable(inspect.isroutine)
print("isroutine_is_callable OK")
