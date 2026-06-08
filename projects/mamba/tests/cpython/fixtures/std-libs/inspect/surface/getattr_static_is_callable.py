# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "getattr_static_is_callable"
# subject = "inspect.getattr_static"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getattr_static: getattr_static_is_callable (surface)."""
import inspect

assert callable(inspect.getattr_static)
print("getattr_static_is_callable OK")
