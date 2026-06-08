# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "variable_annotations"
# dimension = "behavior"
# case = "get_type_hints_resolves_class"
# subject = "typing.get_type_hints"
# kind = "semantic"
# xfail = "get_type_hints reads __annotations__, which is an undefined name on mamba. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.get_type_hints: get_type_hints(C) resolves a class's declared annotations to their runtime type objects: {'val': int}"""
from typing import get_type_hints


class C:
    val: int = 5


hints = get_type_hints(C)
assert "val" in hints, hints
assert hints["val"] is int, hints["val"]
print("get_type_hints_resolves_class OK")
