# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "repr_false_hides_field_keeps_attr"
# subject = "dataclasses.field"
# kind = "semantic"
# xfail = "mamba does not honor field(repr=False) (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "behavior.py"
# status = "filled"
# ///
"""dataclasses.field: field(repr=False) hides a field from __repr__ but keeps it as a real attribute"""
from dataclasses import dataclass, field


@dataclass
class Secret:
    name: str
    token: str = field(repr=False, default="x")


s = Secret("k")
assert "token" not in repr(s)
assert s.token == "x"
print("repr_false_hides_field_keeps_attr OK")
