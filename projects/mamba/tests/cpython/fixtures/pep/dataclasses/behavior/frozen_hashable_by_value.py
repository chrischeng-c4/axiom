# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "dataclasses"
# dimension = "behavior"
# case = "frozen_hashable_by_value"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = "mamba does not synthesize __hash__ for frozen dataclasses (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "behavior.py"
# status = "filled"
# ///
"""dataclasses.dataclass: frozen=True instances expose a stable __hash__ over field value (hash(im) == hash(im))"""
from dataclasses import dataclass


@dataclass(frozen=True)
class Immutable:
    val: int


im = Immutable(5)
assert im.val == 5, f"val = {im.val!r}"
assert hash(im) == hash(im), "frozen hash is stable"
assert hash(Immutable(5)) == hash(Immutable(5)), "frozen hash is by value"
# Usable as a dict key.
table = {Immutable(5): "five"}
assert table[Immutable(5)] == "five", "frozen instance is a usable dict key"
print("frozen_hashable_by_value OK")
