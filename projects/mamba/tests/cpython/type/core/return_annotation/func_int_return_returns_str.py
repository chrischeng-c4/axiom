# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "return_annotation"
# dimension = "type"
# case = "func_int_return_returns_str"
# subject = "function return annotation"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba runtime-type enforcement: function annotated int -> int returns str.

CPython 3.12: accepts the str return.
Mamba: raises TypeError at return time (annotation is a contract).
"""


def a() -> int:
    return "not_an_int"  # type: ignore[return-value]


try:
    result = a()
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
