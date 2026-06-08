# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "annotations_not_runtime_enforced"
# subject = "typing"
# kind = "semantic"
# xfail = "mamba returns None for a function's __annotations__ (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing: annotations are advisory only: _fn(x:int,y:str='hi')->bool has a dict __annotations__ yet calling _fn('not_int') is accepted at runtime and returns True with no type enforcement"""
import typing


def _fn(x: int, y: str = "hi") -> bool:
    return True  # type: ignore[return-value]


assert isinstance(_fn.__annotations__, dict), "annotations dict"
# Annotations are not enforced at runtime: a str is accepted where int is hinted.
assert _fn("not_int") is True, "no runtime enforcement"  # type: ignore[arg-type]

print("annotations_not_runtime_enforced OK")
