# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "frozen_default_hashable_dedups_in_set"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dataclasses.py"
# status = "filled"
# ///
"""dataclasses.dataclass: @dataclass(frozen=True) is hashable by default so equal frozen instances hash equal and deduplicate inside a set"""
import dataclasses


@dataclasses.dataclass(frozen=True)
class FrozenPoint:
    x: int
    y: int


fp = FrozenPoint(1, 2)
assert hash(fp) == hash(FrozenPoint(1, 2)), "frozen hash consistent"
s = {fp, FrozenPoint(1, 2)}
assert len(s) == 1, "frozen deduped in set"

print("frozen_default_hashable_dedups_in_set OK")
