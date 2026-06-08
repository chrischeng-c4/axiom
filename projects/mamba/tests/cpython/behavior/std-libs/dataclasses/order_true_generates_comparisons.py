# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "order_true_generates_comparisons"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dataclasses.py"
# status = "filled"
# ///
"""dataclasses.dataclass: @dataclass(order=True) generates <, >, <= from a lexicographic field-tuple comparison (first field dominates, later fields break ties)"""
import dataclasses


@dataclasses.dataclass(order=True)
class Ordered:
    priority: int
    name: str


o1 = Ordered(1, "a")
o2 = Ordered(2, "b")
o3 = Ordered(1, "b")
assert o1 < o2, "order: priority 1 < 2"
assert o2 > o1, "order: priority 2 > 1"
assert o1 <= o3, "order: priority equal ties to name"

print("order_true_generates_comparisons OK")
