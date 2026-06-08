# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "default_factory_not_shared"
# subject = "dataclasses.field"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dataclasses.py"
# status = "filled"
# ///
"""dataclasses.field: field(default_factory=list) builds a fresh list per instance so mutating one instance's list does not affect another"""
import dataclasses


@dataclasses.dataclass
class WithList:
    items: list = dataclasses.field(default_factory=list)


a = WithList()
b = WithList()
a.items.append(1)
assert b.items == [], f"independent default_factory lists: {b.items!r}"
assert a.items == [1], f"mutated instance keeps its own list: {a.items!r}"

print("default_factory_not_shared OK")
