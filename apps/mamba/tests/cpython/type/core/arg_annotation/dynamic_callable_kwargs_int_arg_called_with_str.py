# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "arg_annotation"
# dimension = "type"
# case = "dynamic_callable_kwargs_int_arg_called_with_str"
# subject = "Any-erased function variadic keyword parameter annotation"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""An Any-erased callable enforces each declared variadic keyword value."""

from typing import Any


def takes_ints(**values: int) -> int:
    return len(values)


dynamic: Any = takes_ints
try:
    result = dynamic(value="wrong")
    print("no_typeerror:", repr(result))
except TypeError as exc:
    print("typeerror:", type(exc).__name__, str(exc)[:80])
