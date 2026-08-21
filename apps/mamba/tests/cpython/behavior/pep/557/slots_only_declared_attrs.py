# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "slots_only_declared_attrs"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = "mamba does not synthesize __slots__ for slots=True (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "frozen_hash_slots.py"
# status = "filled"
# ///
"""dataclasses.dataclass: @dataclass(slots=True) generates __slots__ == declared names and instances have no per-object __dict__"""
from dataclasses import dataclass


@dataclass(slots=True)
class Slotted:
    x: int


sl = Slotted(10)
assert sl.x == 10
sl.x = 11
assert sl.x == 11
assert Slotted.__slots__ == ("x",)
assert not hasattr(sl, "__dict__")  # slotted instances have no per-object dict
print("slots_only_declared_attrs OK")
