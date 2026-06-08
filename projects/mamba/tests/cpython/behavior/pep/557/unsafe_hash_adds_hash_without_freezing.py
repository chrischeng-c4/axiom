# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "unsafe_hash_adds_hash_without_freezing"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = "mamba does not honor unsafe_hash=True (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "frozen_hash_slots.py"
# status = "filled"
# ///
"""dataclasses.dataclass: @dataclass(unsafe_hash=True) adds a __hash__ over the field tuple without freezing"""
from dataclasses import dataclass


@dataclass(unsafe_hash=True)
class H:
    x: int
    y: str


assert hash(H(1, "foo")) == hash((1, "foo"))
print("unsafe_hash_adds_hash_without_freezing OK")
