# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "dataclasses"
# dimension = "behavior"
# case = "replace_overrides_selected_fields"
# subject = "dataclasses.replace"
# kind = "semantic"
# xfail = "mamba does not implement replace() over synthesized dataclasses (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "behavior.py"
# status = "filled"
# ///
"""dataclasses.replace: replace() builds a copy with selected fields overridden and the untouched fields carried over from the original"""
from dataclasses import dataclass, replace


@dataclass
class Vec:
    x: int
    y: int
    z: int = 0


original = Vec(1, 2, 3)
copy = replace(original, z=99)
assert copy.x == 1 and copy.y == 2 and copy.z == 99, f"replace = {copy.x},{copy.y},{copy.z}"
# The original is untouched.
assert original.z == 3, f"original untouched: z = {original.z!r}"
assert copy is not original, "replace returns a new instance"
print("replace_overrides_selected_fields OK")
