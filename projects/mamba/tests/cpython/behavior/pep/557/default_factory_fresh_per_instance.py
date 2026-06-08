# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "default_factory_fresh_per_instance"
# subject = "dataclasses.field"
# kind = "semantic"
# xfail = "mamba does not synthesize default_factory in __init__ (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "field_defaults.py"
# status = "filled"
# ///
"""dataclasses.field: field(default_factory=list) supplies a fresh value per instance (mutable-default safe, not shared)"""
from dataclasses import dataclass, field


@dataclass
class Bag:
    items: list = field(default_factory=list)


a = Bag()
b = Bag()
a.items.append(1)
assert a.items == [1]
assert b.items == []  # not shared
print("default_factory_fresh_per_instance OK")
