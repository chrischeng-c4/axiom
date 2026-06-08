# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "dataclasses"
# dimension = "behavior"
# case = "repr_includes_class_and_fields"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = "mamba does not synthesize __repr__ (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "behavior.py"
# status = "filled"
# ///
"""dataclasses.dataclass: the generated __repr__ includes the class name and renders field=value pairs"""
from dataclasses import dataclass


@dataclass
class Vec:
    x: int
    y: int
    z: int = 0


r = repr(Vec(1, 2, 3))
assert "Vec" in r, f"repr has class name: {r}"
assert "x=1" in r, f"repr has x=1: {r}"
assert "y=2" in r, f"repr has y=2: {r}"
assert "z=3" in r, f"repr has z=3: {r}"
print("repr_includes_class_and_fields OK")
