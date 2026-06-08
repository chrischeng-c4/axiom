# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "make_dataclass_bases_and_options"
# subject = "dataclasses.make_dataclass"
# kind = "semantic"
# xfail = "mamba make_dataclass does not honor bases= or pass-through options (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "make_dataclass.py"
# status = "filled"
# ///
"""dataclasses.make_dataclass: make_dataclass bases= makes the generated class inherit, and decorator options like frozen=True pass through"""
from dataclasses import make_dataclass, FrozenInstanceError


class Base1:
    pass


class Base2:
    pass


E = make_dataclass("E", [("x", int)], bases=(Base1, Base2))
e = E(5)
assert isinstance(e, E)
assert isinstance(e, Base1)
assert isinstance(e, Base2)
assert e.x == 5

F = make_dataclass("F", [("x", int)], frozen=True)
f = F(3)
_raised = False
try:
    f.x = 4
except FrozenInstanceError:
    _raised = True
assert _raised, "expected FrozenInstanceError"
print("make_dataclass_bases_and_options OK")
