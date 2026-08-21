# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "dataclasses"
# dimension = "behavior"
# case = "subclass_adds_fields"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = "mamba does not synthesize inherited dataclass fields (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "behavior.py"
# status = "filled"
# ///
"""dataclasses.dataclass: a @dataclass subclass adds its own fields on top of the base's; the generated __init__ accepts both"""
from dataclasses import dataclass


@dataclass
class Base:
    name: str
    value: int = 0


@dataclass
class Extended(Base):
    extra: str = ""


e = Extended(name="test", value=5, extra="bonus")
assert e.name == "test", f"name = {e.name!r}"
assert e.value == 5, f"value = {e.value!r}"
assert e.extra == "bonus", f"extra = {e.extra!r}"
# Base defaults still apply through the subclass.
e2 = Extended(name="solo")
assert e2.value == 0 and e2.extra == "", f"defaults = {e2.value},{e2.extra!r}"
print("subclass_adds_fields OK")
