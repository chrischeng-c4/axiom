# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "type-strict"
# lib = "arg_annotation"
# dimension = "errors"
# case = "kwargs_int_arg_called_with_str"
# subject = "function variadic keyword parameter annotation"
# kind = "semantic"
# xfail = "variadic keyword annotation type enforcement pending; currently MAMBA_TYPE_LEAKED"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-xfail: variadic keyword annotation type enforcement pending; currently MAMBA_TYPE_LEAKED
# mamba-strict-type: TypeError
"""Mamba rejects a wrong-typed variadic keyword argument annotation."""


def count_items(**items: int) -> int:
    return len(items)


try:
    result = count_items(count="3")  # type: ignore[arg-type]
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
