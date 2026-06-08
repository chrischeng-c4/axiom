# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "dataclasses"
# dimension = "behavior"
# case = "default_factory_fresh_per_instance"
# subject = "dataclasses.field"
# kind = "semantic"
# xfail = "mamba does not synthesize default_factory in __init__ (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "behavior.py"
# status = "filled"
# ///
"""dataclasses.field: field(default_factory=list) supplies a fresh independent value per instance (mutating one does not affect another)"""
from dataclasses import dataclass, field


@dataclass
class Bag:
    items: list = field(default_factory=list)
    count: int = 0


b1 = Bag()
b2 = Bag()
b1.items.append("a")
assert b1.items == ["a"], f"b1 = {b1.items!r}"
assert b2.items == [], f"b2 independent = {b2.items!r}"
assert b1.count == 0 and b2.count == 0, "scalar default applied"
print("default_factory_fresh_per_instance OK")
