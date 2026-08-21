# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "currentframe_is_callable"
# subject = "inspect.currentframe"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.currentframe: currentframe_is_callable (surface)."""
import inspect

assert callable(inspect.currentframe)
print("currentframe_is_callable OK")
