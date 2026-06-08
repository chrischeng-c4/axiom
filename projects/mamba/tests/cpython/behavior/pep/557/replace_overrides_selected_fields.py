# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "replace_overrides_selected_fields"
# subject = "dataclasses.replace"
# kind = "semantic"
# xfail = "mamba does not implement replace() over synthesized dataclasses (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "helpers.py"
# status = "filled"
# ///
"""dataclasses.replace: replace() builds a new instance with selected fields overridden, sharing untouched references; original untouched"""
from dataclasses import dataclass, replace


@dataclass
class Inner:
    token: int
    group: int


@dataclass
class Outer:
    name: str
    id: Inner


v = Outer("Ann", Inner(1, 0))
w = replace(v, name="Bob")
assert w.name == "Bob"
assert w.id is v.id
assert v.name == "Ann"  # original untouched
print("replace_overrides_selected_fields OK")
