# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "errors"
# case = "undeclared_typevar_in_base_raises"
# subject = "typing.TypeVar"
# kind = "mechanical"
# xfail = "an undeclared module-level TypeVar in a new-style generic base does not raise TypeError on mamba (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: undeclared_typevar_in_base_raises (errors)."""
from typing import TypeVar

_raised = False
try:
    exec('S = TypeVar("S")\nclass _M[T](dict[T, S]):\n    ...')
except TypeError:
    _raised = True
assert _raised, "undeclared_typevar_in_base_raises: expected TypeError"
print("undeclared_typevar_in_base_raises OK")
