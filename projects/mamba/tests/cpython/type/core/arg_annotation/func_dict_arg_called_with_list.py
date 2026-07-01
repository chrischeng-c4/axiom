# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "arg_annotation"
# dimension = "type"
# case = "func_dict_arg_called_with_list"
# subject = "function positional parameter annotation"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba runtime-type enforcement: dict-annotated arg called with list.

CPython 3.12: accepts; later .items() would fail but the call lands.
Mamba: raises TypeError at call time.
"""


def keys_of(d: dict) -> int:
    return len(d)


try:
    result = keys_of([1, 2, 3])  # type: ignore[arg-type]
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
