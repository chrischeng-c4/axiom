# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "errors"
# case = "double_generic_base_raises"
# subject = "typing.Generic"
# kind = "mechanical"
# xfail = "class _[T](Generic[T]) does not raise the double-Generic TypeError on mamba (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Generic: double_generic_base_raises (errors)."""
from typing import Generic

_raised = False
try:
    exec('class _D[T](Generic[T]):\n    ...')
except TypeError:
    _raised = True
assert _raised, "double_generic_base_raises: expected TypeError"
print("double_generic_base_raises OK")
