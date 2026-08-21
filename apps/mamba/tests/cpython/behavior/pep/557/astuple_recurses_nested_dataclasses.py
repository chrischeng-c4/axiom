# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "astuple_recurses_nested_dataclasses"
# subject = "dataclasses.astuple"
# kind = "semantic"
# xfail = "mamba does not implement astuple over synthesized dataclasses (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "helpers.py"
# status = "filled"
# ///
"""dataclasses.astuple: astuple recurses into nested dataclasses yielding nested tuples; result type is tuple, fresh each call"""
from dataclasses import dataclass, astuple


@dataclass
class Inner:
    token: int
    group: int


@dataclass
class Outer:
    name: str
    id: Inner


v = Outer("Ann", Inner(1, 0))
assert astuple(v) == ("Ann", (1, 0))
assert type(astuple(v)) is tuple
assert astuple(v) is not astuple(v)
print("astuple_recurses_nested_dataclasses OK")
