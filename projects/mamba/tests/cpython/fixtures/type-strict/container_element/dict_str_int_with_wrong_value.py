# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "type-strict"
# lib = "container_element"
# dimension = "errors"
# case = "dict_str_int_with_wrong_value"
# subject = "dict value annotation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba runtime-type enforcement: dict[str, int]-annotated var with wrong value.

CPython 3.12: dict contains the wrong-typed value; annotation
ignored at runtime.
Mamba: raises TypeError because a value type violates the
container annotation contract.
"""

try:
    d: dict[str, int] = {"a": "not_an_int"}  # type: ignore[dict-item]
    print("no_typeerror:", repr(d))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
