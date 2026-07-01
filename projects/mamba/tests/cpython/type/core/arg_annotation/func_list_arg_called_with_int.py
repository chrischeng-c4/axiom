# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "arg_annotation"
# dimension = "type"
# case = "func_list_arg_called_with_int"
# subject = "function positional parameter annotation"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba runtime-type enforcement: list-annotated arg called with int.

CPython 3.12: accepts; uses the int wherever the list would be used
(crash deferred until the body tries a list operation).
Mamba: raises TypeError at call time.
"""


def take(lst: list) -> int:
    # Don't touch lst — keep the body free of operations that raise
    # on the wrong-typed value. We're testing annotation enforcement
    # at call time, not body-level failure.
    return 0


try:
    result = take(7)  # type: ignore[arg-type]
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
