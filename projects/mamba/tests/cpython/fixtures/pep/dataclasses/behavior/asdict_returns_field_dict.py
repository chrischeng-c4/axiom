# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "dataclasses"
# dimension = "behavior"
# case = "asdict_returns_field_dict"
# subject = "dataclasses.asdict"
# kind = "semantic"
# xfail = "mamba does not implement asdict over synthesized dataclasses (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "surface.py"
# status = "filled"
# ///
"""dataclasses.asdict: asdict(instance) returns a plain dict mapping field name to value ({'x': 3.0, 'y': 4.0})"""
from dataclasses import dataclass, asdict


@dataclass
class Point:
    x: float
    y: float


d = asdict(Point(3.0, 4.0))
assert isinstance(d, dict), f"asdict type = {type(d)!r}"
assert d == {"x": 3.0, "y": 4.0}, f"asdict = {d!r}"
print("asdict_returns_field_dict OK")
