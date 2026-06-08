# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "dataclasses"
# dimension = "behavior"
# case = "eq_compares_field_by_field"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = "mamba does not synthesize value-based __eq__ (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "behavior.py"
# status = "filled"
# ///
"""dataclasses.dataclass: the generated __eq__ compares field by field (equal vecs are ==, differing vecs are !=)"""
from dataclasses import dataclass


@dataclass
class Vec:
    x: int
    y: int
    z: int = 0


assert Vec(1, 2, 3) == Vec(1, 2, 3), "equal vecs compare =="
assert Vec(1, 2, 3) != Vec(1, 2, 4), "differing vecs compare !="
assert not (Vec(1, 2, 3) == Vec(1, 2, 4)), "differing vecs are not =="
print("eq_compares_field_by_field OK")
