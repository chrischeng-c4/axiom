# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "isabstract_is_callable"
# subject = "inspect.isabstract"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.isabstract: isabstract_is_callable (surface)."""
import inspect

assert callable(inspect.isabstract)
print("isabstract_is_callable OK")
