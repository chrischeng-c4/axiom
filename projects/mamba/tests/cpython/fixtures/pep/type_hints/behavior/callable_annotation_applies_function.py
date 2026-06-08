# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "callable_annotation_applies_function"
# subject = "typing.Callable"
# kind = "semantic"
# xfail = "mamba diverges on the typing Callable runtime machinery (project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Callable: a Callable[[int],int]-annotated parameter is just a function at runtime: _apply(lambda x: x*2, 5)==10"""
import typing
from typing import Callable


def _apply(fn: Callable[[int], int], v: int) -> int:
    return fn(v)


assert _apply(lambda x: x * 2, 5) == 10, f"apply = {_apply(lambda x: x * 2, 5)!r}"

print("callable_annotation_applies_function OK")
