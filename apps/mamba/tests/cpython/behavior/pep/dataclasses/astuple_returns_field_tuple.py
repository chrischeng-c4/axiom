# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "dataclasses"
# dimension = "behavior"
# case = "astuple_returns_field_tuple"
# subject = "dataclasses.astuple"
# kind = "semantic"
# xfail = "mamba does not implement astuple over synthesized dataclasses (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "surface.py"
# status = "filled"
# ///
"""dataclasses.astuple: astuple(instance) returns a plain tuple of field values in declared order ((3.0, 4.0))"""
from dataclasses import dataclass, astuple


@dataclass
class Point:
    x: float
    y: float


t = astuple(Point(3.0, 4.0))
assert isinstance(t, tuple), f"astuple type = {type(t)!r}"
assert t == (3.0, 4.0), f"astuple = {t!r}"
print("astuple_returns_field_tuple OK")
