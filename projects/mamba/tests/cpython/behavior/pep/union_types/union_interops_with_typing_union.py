# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "union_types"
# dimension = "behavior"
# case = "union_interops_with_typing_union"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | str` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: isinstance over an X | Y union and over typing.Union[X, Y] agree on the same members"""
import types
from typing import Union

pipe = int | str
classic = Union[int, str]
for value in (42, "hi", 3.14):
    assert isinstance(value, pipe) == isinstance(value, classic), value

print("union_interops_with_typing_union OK")
