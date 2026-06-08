# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "errors"
# case = "mutable_dataclass_unhashable_raises"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = "mamba does not remove __hash__ for mutable dataclasses (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "frozen_hash_slots.py"
# status = "filled"
# ///
"""dataclasses.dataclass: a plain (eq=True, frozen=False) dataclass sets __hash__ to None, so hash() raises TypeError"""
from dataclasses import dataclass


@dataclass
class Mutable:
    x: int


_raised = False
try:
    hash(Mutable(1))
except TypeError:
    _raised = True
assert _raised, "mutable_dataclass_unhashable_raises: expected TypeError"
print("mutable_dataclass_unhashable_raises OK")
