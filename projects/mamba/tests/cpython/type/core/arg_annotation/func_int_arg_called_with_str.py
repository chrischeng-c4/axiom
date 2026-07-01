# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "arg_annotation"
# dimension = "type"
# case = "func_int_arg_called_with_str"
# subject = "function positional parameter annotation"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba runtime-type enforcement: int-annotated arg called with str.

CPython 3.12: annotations are documentation, call succeeds.
Mamba:        annotations are contract, call raises TypeError.
"""


def a(i: int) -> int:
    return i


try:
    result = a("a")  # type: ignore[arg-type]
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
