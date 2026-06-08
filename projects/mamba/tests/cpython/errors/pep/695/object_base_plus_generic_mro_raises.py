# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "errors"
# case = "object_base_plus_generic_mro_raises"
# subject = "typing.Generic"
# kind = "mechanical"
# xfail = "class _[X](object) does not raise the object+Generic MRO TypeError on mamba (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Generic: object_base_plus_generic_mro_raises (errors)."""
from typing import Generic

_raised = False
try:
    exec('class _W[X](object):\n    ...')
except TypeError:
    _raised = True
assert _raised, "object_base_plus_generic_mro_raises: expected TypeError"
print("object_base_plus_generic_mro_raises OK")
