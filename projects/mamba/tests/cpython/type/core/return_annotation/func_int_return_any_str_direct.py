# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "return_annotation"
# dimension = "type"
# case = "func_int_return_any_str_direct"
# subject = "synchronous function scalar return annotation"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""A declared scalar return rejects an Any-origin mismatch before direct callers consume it."""

from typing import Any

type Count = int


def bad_return() -> Count:
    value: Any = "wrong"
    print("BODY_EXECUTED")
    return value


try:
    result = bad_return()
    print("CALLER_CONSUMED", repr(result))
except TypeError as exc:
    print("typeerror:", type(exc).__name__, str(exc))
