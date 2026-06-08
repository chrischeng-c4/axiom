# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "typevar_identity_not_enforced"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = "mamba diverges on the typing TypeVar/runtime machinery (project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: a TypeVar is an unenforced placeholder: T=TypeVar('T'); _identity(x:T)->T returns 42 for 42 and 'hi' for 'hi', and isinstance(T, TypeVar) is True"""
import typing
from typing import TypeVar

T = TypeVar("T")


def _identity(x: T) -> T:
    return x


assert _identity(42) == 42, f"identity int = {_identity(42)!r}"
assert _identity("hi") == "hi", f"identity str = {_identity('hi')!r}"
assert isinstance(T, TypeVar), f"T is TypeVar = {type(T)!r}"

print("typevar_identity_not_enforced OK")
