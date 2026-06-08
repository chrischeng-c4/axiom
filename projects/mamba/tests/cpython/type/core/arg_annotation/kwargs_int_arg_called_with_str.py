# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "arg_annotation"
# dimension = "type"
# case = "kwargs_int_arg_called_with_str"
# subject = "function variadic keyword parameter annotation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba rejects a wrong-typed variadic keyword argument annotation."""


def count_items(**items: int) -> int:
    return len(items)


try:
    result = count_items(count="3")  # type: ignore[arg-type]
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
