# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "get_type_hints_returns_param_dict"
# subject = "typing.get_type_hints"
# kind = "semantic"
# xfail = "mamba diverges on the typing get_type_hints runtime machinery (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.get_type_hints: get_type_hints returns a dict of resolved annotations: for _typed(a:int,b:Optional[str]=None)->List[int] it is a dict containing 'a', 'b' and 'return'"""
import typing
from typing import List, Optional


def _typed(a: int, b: Optional[str] = None) -> List[int]:
    return []


_hints = typing.get_type_hints(_typed)
assert isinstance(_hints, dict), f"hints type = {type(_hints)!r}"
assert "a" in _hints, "a in hints"
assert "b" in _hints, "b in hints"
assert "return" in _hints, "return in hints"

print("get_type_hints_returns_param_dict OK")
