# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "errors"
# case = "frozen_setattr_raises"
# subject = "dataclasses.FrozenInstanceError"
# kind = "semantic"
# xfail = "mamba does not honor frozen synthesized __setattr__ (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "errors.py"
# status = "filled"
# ///
"""dataclasses.FrozenInstanceError: assigning to a field of a frozen=True instance raises FrozenInstanceError"""
from dataclasses import dataclass, FrozenInstanceError


@dataclass(frozen=True)
class P:
    x: int
    y: int


p = P(1, 2)
_raised = False
try:
    p.x = 10  # type: ignore[misc]
except FrozenInstanceError:
    _raised = True
assert _raised, "frozen_setattr_raises: expected FrozenInstanceError"
print("frozen_setattr_raises OK")
