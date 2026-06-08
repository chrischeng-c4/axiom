# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "order_true_generates_rich_comparison"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = "mamba does not synthesize ordering dunders for order=True (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "behavior.py"
# status = "filled"
# ///
"""dataclasses.dataclass: @dataclass(order=True) generates rich comparisons ordering by the field tuple, so instances sort"""
from dataclasses import dataclass


@dataclass(order=True)
class Ver:
    major: int
    minor: int


assert Ver(1, 0) < Ver(1, 5)
assert Ver(2, 0) > Ver(1, 9)
assert sorted([Ver(2, 0), Ver(1, 0), Ver(1, 5)]) == [Ver(1, 0), Ver(1, 5), Ver(2, 0)]
print("order_true_generates_rich_comparison OK")
