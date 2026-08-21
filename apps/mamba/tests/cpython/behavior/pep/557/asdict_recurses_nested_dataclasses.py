# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "asdict_recurses_nested_dataclasses"
# subject = "dataclasses.asdict"
# kind = "semantic"
# xfail = "mamba does not implement asdict over synthesized dataclasses (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "helpers.py"
# status = "filled"
# ///
"""dataclasses.asdict: asdict recurses into nested dataclasses producing plain dicts, and builds a fresh deep copy each call"""
from dataclasses import dataclass, asdict


@dataclass
class Inner:
    token: int
    group: int


@dataclass
class Outer:
    name: str
    id: Inner


u = Outer("Joe", Inner(123, 1))
assert asdict(u) == {"name": "Joe", "id": {"token": 123, "group": 1}}
assert asdict(u) is not asdict(u)  # fresh structure each call
u.id.group = 2
assert asdict(u) == {"name": "Joe", "id": {"token": 123, "group": 2}}
print("asdict_recurses_nested_dataclasses OK")
