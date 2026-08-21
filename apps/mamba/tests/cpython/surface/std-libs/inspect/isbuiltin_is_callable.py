# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "isbuiltin_is_callable"
# subject = "inspect.isbuiltin"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.isbuiltin: isbuiltin_is_callable (surface)."""
import inspect

assert callable(inspect.isbuiltin)
print("isbuiltin_is_callable OK")
