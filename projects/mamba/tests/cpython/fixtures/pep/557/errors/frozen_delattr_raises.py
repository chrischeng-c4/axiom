# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "errors"
# case = "frozen_delattr_raises"
# subject = "dataclasses.FrozenInstanceError"
# kind = "semantic"
# xfail = "mamba does not honor frozen synthesized __delattr__ (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "frozen_hash_slots.py"
# status = "filled"
# ///
"""dataclasses.FrozenInstanceError: deleting a field of a frozen=True instance raises FrozenInstanceError"""
from dataclasses import dataclass, FrozenInstanceError


@dataclass(frozen=True)
class Coord:
    x: int
    y: int


c = Coord(1, 2)
_raised = False
try:
    del c.x
except FrozenInstanceError:
    _raised = True
assert _raised, "frozen_delattr_raises: expected FrozenInstanceError"
print("frozen_delattr_raises OK")
