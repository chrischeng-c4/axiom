# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "get_type_hints_resolves_function"
# subject = "typing.get_type_hints"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.get_type_hints: get_type_hints on a function returns a dict mapping each parameter and 'return' to its resolved runtime type (a: int, b: str, return: bool)"""
import typing


def annotated(a: int, b: str) -> bool:
    return bool(a) and bool(b)


hints = typing.get_type_hints(annotated)
assert hints == {"a": int, "b": str, "return": bool}, f"resolved hints = {hints!r}"
print("get_type_hints_resolves_function OK")
