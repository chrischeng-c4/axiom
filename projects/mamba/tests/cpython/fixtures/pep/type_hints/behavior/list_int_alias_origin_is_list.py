# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "list_int_alias_origin_is_list"
# subject = "typing.List"
# kind = "semantic"
# xfail = "mamba diverges on the typing generic-alias runtime machinery (project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.List: List[int] is a generic alias exposing its origin: hasattr(List[int],'__origin__') is True and List[int].__origin__ is the builtin list"""
import typing
from typing import List

_IntList = List[int]
assert hasattr(_IntList, "__origin__"), "List[int] has __origin__"
assert _IntList.__origin__ is list, f"__origin__ = {_IntList.__origin__!r}"

print("list_int_alias_origin_is_list OK")
