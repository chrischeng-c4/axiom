# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "arg_annotation"
# dimension = "type"
# case = "dynamic_callable_int_arg_called_with_str"
# subject = "Any-erased function parameter annotation"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""An Any-erased callable still enforces its declared int parameter."""

from typing import Any


def takes_int(value: int) -> int:
    return value


dynamic: Any = takes_int
try:
    result = dynamic("wrong")
    print("no_typeerror:", repr(result))
except TypeError as exc:
    print("typeerror:", type(exc).__name__, str(exc)[:80])
