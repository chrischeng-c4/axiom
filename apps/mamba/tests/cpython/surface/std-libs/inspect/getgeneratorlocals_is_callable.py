# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "getgeneratorlocals_is_callable"
# subject = "inspect.getgeneratorlocals"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getgeneratorlocals: getgeneratorlocals_is_callable (surface)."""
import inspect

assert callable(inspect.getgeneratorlocals)
print("getgeneratorlocals_is_callable OK")
