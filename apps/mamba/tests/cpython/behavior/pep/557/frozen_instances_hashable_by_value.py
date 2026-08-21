# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "frozen_instances_hashable_by_value"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = "mamba does not synthesize __hash__ for frozen dataclasses (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "frozen_hash_slots.py"
# status = "filled"
# ///
"""dataclasses.dataclass: frozen=True instances are immutable and hashable by field value, usable as dict keys"""
from dataclasses import dataclass


@dataclass(frozen=True)
class Coord:
    x: int
    y: int


c = Coord(1, 2)
assert hash(c) == hash(Coord(1, 2))
assert {Coord(1, 2): "a"}[Coord(1, 2)] == "a"  # usable as dict key
print("frozen_instances_hashable_by_value OK")
