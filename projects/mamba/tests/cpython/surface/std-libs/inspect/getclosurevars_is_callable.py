# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "getclosurevars_is_callable"
# subject = "inspect.getclosurevars"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getclosurevars: getclosurevars_is_callable (surface)."""
import inspect

assert callable(inspect.getclosurevars)
print("getclosurevars_is_callable OK")
