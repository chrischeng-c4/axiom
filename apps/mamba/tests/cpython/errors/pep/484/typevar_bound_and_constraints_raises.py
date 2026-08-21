# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "errors"
# case = "typevar_bound_and_constraints_raises"
# subject = "typing.TypeVar"
# kind = "mechanical"
# xfail = "mamba does not raise when a TypeVar mixes constraints with bound= (project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: typevar_bound_and_constraints_raises (errors)."""
import typing

_raised = False
try:
    typing.TypeVar('T', int, str, bound=int)
except TypeError:
    _raised = True
assert _raised, "typevar_bound_and_constraints_raises: expected TypeError"
print("typevar_bound_and_constraints_raises OK")
