# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "type-strict"
# lib = "container_element"
# dimension = "errors"
# case = "list_int_with_str_element"
# subject = "list element annotation"
# kind = "semantic"
# xfail = "list element annotation enforcement pending; currently MAMBA_TYPE_LEAKED"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-xfail: list element annotation enforcement pending; currently MAMBA_TYPE_LEAKED
# mamba-strict-type: TypeError
"""Mamba runtime-type enforcement: list[int]-annotated var with str element.

CPython 3.12: annotation is documentation, list contains the str.
Mamba: raises TypeError because the element type violates the
container annotation contract.
"""

try:
    xs: list[int] = [1, "two", 3]  # type: ignore[list-item]
    print("no_typeerror:", repr(xs))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
