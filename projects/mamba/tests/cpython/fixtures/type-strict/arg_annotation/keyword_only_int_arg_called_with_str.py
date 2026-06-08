# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "type-strict"
# lib = "arg_annotation"
# dimension = "errors"
# case = "keyword_only_int_arg_called_with_str"
# subject = "function keyword-only parameter annotation"
# kind = "semantic"
# xfail = "keyword-only annotation type enforcement pending; currently MAMBA_TYPE_LEAKED"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-xfail: keyword-only annotation type enforcement pending; currently MAMBA_TYPE_LEAKED
# mamba-strict-type: TypeError
"""Mamba rejects a wrong-typed keyword-only argument annotation."""


def requires_count(*, count: int) -> int:
    return count


try:
    result = requires_count(count="3")  # type: ignore[arg-type]
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
