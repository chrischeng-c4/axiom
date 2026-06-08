# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "type-strict"
# lib = "arg_annotation"
# dimension = "errors"
# case = "default_int_arg_uses_str_default"
# subject = "function default parameter annotation"
# kind = "semantic"
# xfail = "default argument annotation type enforcement pending; currently MAMBA_TYPE_LEAKED"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-xfail: default argument annotation type enforcement pending; currently MAMBA_TYPE_LEAKED
# mamba-strict-type: TypeError
"""Mamba rejects an annotated int parameter whose default is a str."""


def requires_count(count: int = "3") -> int:  # type: ignore[assignment]
    return count


try:
    result = requires_count()
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
