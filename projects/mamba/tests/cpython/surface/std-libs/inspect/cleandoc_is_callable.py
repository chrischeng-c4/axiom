# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "cleandoc_is_callable"
# subject = "inspect.cleandoc"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.cleandoc: cleandoc_is_callable (surface)."""
import inspect

assert callable(inspect.cleandoc)
print("cleandoc_is_callable OK")
