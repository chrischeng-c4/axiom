# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "dataclasses"
# dimension = "behavior"
# case = "fields_lists_field_names"
# subject = "dataclasses.fields"
# kind = "semantic"
# xfail = "mamba does not expose synthesized dataclass fields (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "behavior.py"
# status = "filled"
# ///
"""dataclasses.fields: fields(C) returns Field objects whose .name covers every declared field in declared order"""
from dataclasses import dataclass, fields


@dataclass
class Vec:
    x: int
    y: int
    z: int = 0


names = [f.name for f in fields(Vec)]
assert names == ["x", "y", "z"], f"field names = {names!r}"
# Same result whether queried by class or by instance.
assert [f.name for f in fields(Vec(1, 2, 3))] == names, "fields() agrees for class and instance"
print("fields_lists_field_names OK")
