# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "union_dedups_and_flattens"
# subject = "typing.Union"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Union: Union deduplicates members, ignores order, collapses a single member, and flattens nested unions: Union[int,int] is int, Union[int,str]==Union[str,int], Union[int,str,int]==Union[int,str], Union[Union[int,str],float]==Union[int,str,float], and Union[int,float]!=Union"""
from typing import Optional, Union

# Union deduplicates members and ignores order.
assert Union[int, int] is int
assert Union[int, str] == Union[str, int]
assert Union[int, str, int] == Union[int, str]
# Nested unions flatten into a single union.
assert Union[Union[int, str], float] == Union[int, str, float]
# Optional[X] is Union[X, None]; the bare special form is not a union value.
assert Optional[int] == Union[int, None]
assert Union[int, float] != Union

print("union_dedups_and_flattens OK")
