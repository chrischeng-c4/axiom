# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "fields_same_for_class_and_instance"
# subject = "dataclasses.fields"
# kind = "semantic"
# xfail = "mamba does not expose synthesized dataclass fields (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "helpers.py"
# status = "filled"
# ///
"""dataclasses.fields: fields() returns the same metadata whether queried by class or by instance, in declared order"""
from dataclasses import dataclass, fields


@dataclass
class Inner:
    token: int
    group: int


@dataclass
class Outer:
    name: str
    id: int


assert fields(Inner) == fields(Inner(0, 0))
assert tuple(f.name for f in fields(Outer)) == ("name", "id")
print("fields_same_for_class_and_instance OK")
