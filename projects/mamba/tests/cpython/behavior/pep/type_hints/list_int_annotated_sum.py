# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "list_int_annotated_sum"
# subject = "typing.List"
# kind = "semantic"
# xfail = "mamba diverges on the typing generic-alias runtime machinery (project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.List: a List[int]-annotated function works on a normal list: _sum_list(items:List[int])->int returns 6 for [1,2,3]"""
import typing
from typing import List


def _sum_list(items: List[int]) -> int:
    return sum(items)


assert _sum_list([1, 2, 3]) == 6, f"sum_list = {_sum_list([1, 2, 3])!r}"

print("list_int_annotated_sum OK")
