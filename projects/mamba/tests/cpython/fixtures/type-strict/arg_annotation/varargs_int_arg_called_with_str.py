# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "type-strict"
# lib = "arg_annotation"
# dimension = "errors"
# case = "varargs_int_arg_called_with_str"
# subject = "function variadic positional parameter annotation"
# kind = "semantic"
# xfail = "variadic positional annotation type enforcement pending; currently MAMBA_TYPE_LEAKED"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-xfail: variadic positional annotation type enforcement pending; currently MAMBA_TYPE_LEAKED
# mamba-strict-type: TypeError
"""Mamba rejects a wrong-typed variadic positional argument annotation."""


def sum_items(*items: int) -> int:
    return len(items)


try:
    result = sum_items("3")  # type: ignore[arg-type]
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
