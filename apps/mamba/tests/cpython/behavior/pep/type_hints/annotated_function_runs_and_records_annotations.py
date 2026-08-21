# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "annotated_function_runs_and_records_annotations"
# subject = "typing.get_type_hints"
# kind = "semantic"
# xfail = "mamba returns None for a function's __annotations__ (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.get_type_hints: an int->int annotated function runs normally and stores its hints: _add(a:int,b:int)->int returns 5 for (2,3), is callable, and 'return' is in _add.__annotations__"""
import typing


def _add(a: int, b: int) -> int:
    return a + b


assert callable(_add), "_add callable"
assert _add(2, 3) == 5, f"add = {_add(2, 3)!r}"
assert "return" in _add.__annotations__, "return in annotations"

print("annotated_function_runs_and_records_annotations OK")
