# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "classvar_excluded_from_init_and_fields"
# subject = "dataclasses.fields"
# kind = "semantic"
# xfail = "mamba does not exclude ClassVar from synthesized dataclass fields (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "field_defaults.py"
# status = "filled"
# ///
"""dataclasses.fields: a ClassVar annotation is excluded from __init__ and from fields(), and stays shared on the class"""
from dataclasses import dataclass, fields
from typing import ClassVar


@dataclass
class WithClassVar:
    x: int
    kind: ClassVar[str] = "default"


obj = WithClassVar(1)
assert obj.x == 1
assert obj.kind == "default"
assert [f.name for f in fields(WithClassVar)] == ["x"]
assert WithClassVar(1).kind is WithClassVar(2).kind  # shared on the class
print("classvar_excluded_from_init_and_fields OK")
