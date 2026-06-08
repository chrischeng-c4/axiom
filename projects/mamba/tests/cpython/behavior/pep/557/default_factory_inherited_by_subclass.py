# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "default_factory_inherited_by_subclass"
# subject = "dataclasses.field"
# kind = "semantic"
# xfail = "mamba does not synthesize inherited dataclass fields (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "field_defaults.py"
# status = "filled"
# ///
"""dataclasses.field: a subclass inherits the base's default_factory field behavior"""
from dataclasses import dataclass, field


@dataclass
class Counted:
    n: int = field(default_factory=lambda: 0)


@dataclass
class Derived(Counted):
    extra: int = 7


assert Derived(99).extra == 7
assert Derived(99).n == 99
print("default_factory_inherited_by_subclass OK")
