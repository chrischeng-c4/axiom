# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "errors"
# case = "generic_too_many_params_raises"
# subject = "typing.List"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.List: generic_too_many_params_raises (errors)."""
import typing

_raised = False
try:
    typing.List[int, str, float]
except TypeError:
    _raised = True
assert _raised, "generic_too_many_params_raises: expected TypeError"
print("generic_too_many_params_raises OK")
