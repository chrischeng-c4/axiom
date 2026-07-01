# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "return_annotation"
# dimension = "type"
# case = "func_list_return_returns_int"
# subject = "function return annotation"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba runtime-type enforcement: list-return function returns int.

CPython 3.12: accepts the int return.
Mamba: raises TypeError at return time.
"""


def get() -> list:
    return 7  # type: ignore[return-value]


try:
    result = get()
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
