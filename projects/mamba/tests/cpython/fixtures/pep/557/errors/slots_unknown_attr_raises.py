# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "errors"
# case = "slots_unknown_attr_raises"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = "mamba does not synthesize __slots__ for slots=True dataclasses (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "frozen_hash_slots.py"
# status = "filled"
# ///
"""dataclasses.dataclass: assigning an undeclared attribute on a slots=True instance raises AttributeError"""
from dataclasses import dataclass


@dataclass(slots=True)
class Slotted:
    x: int


sl = Slotted(10)
_raised = False
try:
    sl.y = 5  # not a declared slot
except AttributeError:
    _raised = True
assert _raised, "slots_unknown_attr_raises: expected AttributeError"
print("slots_unknown_attr_raises OK")
