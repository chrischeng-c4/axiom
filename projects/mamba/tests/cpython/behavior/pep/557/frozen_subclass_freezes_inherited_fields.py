# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "frozen_subclass_freezes_inherited_fields"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = "mamba does not synthesize frozen __setattr__ for subclasses (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "frozen_hash_slots.py"
# status = "filled"
# ///
"""dataclasses.dataclass: a frozen subclass of a frozen base freezes inherited fields too"""
from dataclasses import dataclass, FrozenInstanceError


@dataclass(frozen=True)
class Coord:
    x: int
    y: int


@dataclass(frozen=True)
class Coord3(Coord):
    z: int


d = Coord3(0, 1, 2)
assert (d.x, d.y, d.z) == (0, 1, 2)
_raised = False
try:
    d.x = 5  # inherited field still frozen
except FrozenInstanceError:
    _raised = True
assert _raised, "expected FrozenInstanceError on inherited field"
print("frozen_subclass_freezes_inherited_fields OK")
