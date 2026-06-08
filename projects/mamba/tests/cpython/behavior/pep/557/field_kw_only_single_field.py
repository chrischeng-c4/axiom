# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "field_kw_only_single_field"
# subject = "dataclasses.field"
# kind = "semantic"
# xfail = "mamba does not honor field(kw_only=True) (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "keyword_only.py"
# status = "filled"
# ///
"""dataclasses.field: field(kw_only=True) opts a single field into keyword-only"""
from dataclasses import dataclass, field


@dataclass
class Mixed:
    a: int
    b: int = field(kw_only=True)


m = Mixed(1, b=2)
assert (m.a, m.b) == (1, 2)
print("field_kw_only_single_field OK")
