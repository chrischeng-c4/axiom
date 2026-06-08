# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "errors"
# case = "typevar_bound_and_constraints_raises"
# subject = "typing.TypeVar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.TypeVar: typevar_bound_and_constraints_raises (errors)."""
import typing

_raised = False
try:
    typing.TypeVar("T", int, str, bound=int)
except TypeError:
    _raised = True
assert _raised, "typevar_bound_and_constraints_raises: expected TypeError"
print("typevar_bound_and_constraints_raises OK")
