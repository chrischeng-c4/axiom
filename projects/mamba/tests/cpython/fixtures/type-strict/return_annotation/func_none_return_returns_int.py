# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "type-strict"
# lib = "return_annotation"
# dimension = "errors"
# case = "func_none_return_returns_int"
# subject = "function None return annotation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba rejects returning a value from a function annotated as None."""


def a() -> None:
    return 7  # type: ignore[return-value]


try:
    result = a()
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
