# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "dataclasses"
# dimension = "behavior"
# case = "order_true_generates_comparisons"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = "mamba does not synthesize ordering dunders for order=True (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "behavior.py"
# status = "filled"
# ///
"""dataclasses.dataclass: @dataclass(order=True) generates rich comparisons ordering by the field tuple (< > <= behave like the field)"""
from dataclasses import dataclass


@dataclass(order=True)
class Score:
    value: int


assert Score(3) < Score(5), "order lt"
assert Score(5) > Score(3), "order gt"
assert Score(3) <= Score(3), "order le (equal)"
assert Score(5) >= Score(3), "order ge"
# Sortable.
ordered = sorted([Score(5), Score(1), Score(3)])
assert [s.value for s in ordered] == [1, 3, 5], f"sorted = {[s.value for s in ordered]!r}"
print("order_true_generates_comparisons OK")
