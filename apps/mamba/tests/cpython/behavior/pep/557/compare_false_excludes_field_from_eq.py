# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "compare_false_excludes_field_from_eq"
# subject = "dataclasses.field"
# kind = "semantic"
# xfail = "mamba does not honor field(compare=False) (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "behavior.py"
# status = "filled"
# ///
"""dataclasses.field: field(compare=False) excludes that field from the generated __eq__"""
from dataclasses import dataclass, field


@dataclass
class Tagged:
    x: int = 0
    note: str = field(compare=False, default="")


assert Tagged(1, "a") == Tagged(1, "b")
assert Tagged(1) != Tagged(2)
print("compare_false_excludes_field_from_eq OK")
