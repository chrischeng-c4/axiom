# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "ismodule_is_callable"
# subject = "inspect.ismodule"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.ismodule: ismodule_is_callable (surface)."""
import inspect

assert callable(inspect.ismodule)
print("ismodule_is_callable OK")
