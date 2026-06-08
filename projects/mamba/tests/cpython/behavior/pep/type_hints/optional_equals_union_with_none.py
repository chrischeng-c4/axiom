# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "optional_equals_union_with_none"
# subject = "typing.Optional"
# kind = "semantic"
# xfail = "mamba diverges on the typing union/| machinery (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Optional: Optional[X] is exactly Union[X,None]: typing.Optional[int]==Union[int,None] and typing.Optional[str]==Union[str,None]"""
import typing
from typing import Optional, Union

assert Optional[int] == Union[int, None], "Optional[int] == Union[int, None]"
assert Optional[str] == Union[str, None], "Optional[str] == Union[str, None]"

print("optional_equals_union_with_none OK")
