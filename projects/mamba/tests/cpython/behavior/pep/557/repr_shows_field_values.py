# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "repr_shows_field_values"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = "mamba does not synthesize __repr__ (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "behavior.py"
# status = "filled"
# ///
"""dataclasses.dataclass: the generated __repr__ renders ClassName(field=value, ...) in field order"""
from dataclasses import dataclass


@dataclass
class Point:
    x: int
    y: int = 0


assert repr(Point(1, 2)) == "Point(x=1, y=2)"
print("repr_shows_field_values OK")
