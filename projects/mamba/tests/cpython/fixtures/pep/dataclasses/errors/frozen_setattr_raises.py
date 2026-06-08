# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "dataclasses"
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
class Point:
    x: int
    y: int


p = Point(1, 2)
assert p.x == 1 and p.y == 2, f"point = {p.x},{p.y}"

_raised = False
try:
    p.x = 9  # type: ignore[misc]
except FrozenInstanceError:
    _raised = True
assert _raised, "frozen_setattr_raises: expected FrozenInstanceError"
print("frozen_setattr_raises OK")
