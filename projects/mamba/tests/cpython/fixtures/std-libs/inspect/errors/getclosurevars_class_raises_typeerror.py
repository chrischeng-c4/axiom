# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "errors"
# case = "getclosurevars_class_raises_typeerror"
# subject = "inspect.getclosurevars"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getclosurevars: getclosurevars_class_raises_typeerror (errors)."""
import inspect

_raised = False
try:
    inspect.getclosurevars(list)
except TypeError:
    _raised = True
assert _raised, "getclosurevars_class_raises_typeerror: expected TypeError"
print("getclosurevars_class_raises_typeerror OK")
