# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "eq_compares_by_field_value"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = "mamba does not synthesize value-based __eq__ (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "behavior.py"
# status = "filled"
# ///
"""dataclasses.dataclass: the generated __eq__ compares by field value (not identity) and only between same-type instances"""
from dataclasses import dataclass


@dataclass
class Point:
    x: int
    y: int = 0


assert Point(1, 2) == Point(1, 2)
assert Point(1, 2) != Point(1, 3)
assert (Point(1, 2) == (1, 2)) is False
print("eq_compares_by_field_value OK")
