# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "union_annotation_accepts_either_member"
# subject = "typing.Union"
# kind = "semantic"
# xfail = "mamba diverges on the typing union/| machinery (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Union: a Union[int,str]-annotated function accepts either member at runtime: _first(1)=='1' and _first('a')=='a' (annotation is advisory)"""
import typing
from typing import Union


def _first(v: Union[int, str]) -> str:
    return str(v)


assert _first(1) == "1", f"first(1) = {_first(1)!r}"
assert _first("a") == "a", f"first(a) = {_first('a')!r}"

print("union_annotation_accepts_either_member OK")
