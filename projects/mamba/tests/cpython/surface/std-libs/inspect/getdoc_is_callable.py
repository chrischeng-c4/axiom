# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "getdoc_is_callable"
# subject = "inspect.getdoc"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getdoc: getdoc_is_callable (surface)."""
import inspect

assert callable(inspect.getdoc)
print("getdoc_is_callable OK")
