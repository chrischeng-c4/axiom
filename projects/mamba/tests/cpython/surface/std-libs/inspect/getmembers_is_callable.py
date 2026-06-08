# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "getmembers_is_callable"
# subject = "inspect.getmembers"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getmembers: getmembers_is_callable (surface)."""
import inspect

assert callable(inspect.getmembers)
print("getmembers_is_callable OK")
