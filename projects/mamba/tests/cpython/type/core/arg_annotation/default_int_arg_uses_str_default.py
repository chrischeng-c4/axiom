# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "arg_annotation"
# dimension = "type"
# case = "default_int_arg_uses_str_default"
# subject = "function default parameter annotation"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba rejects an annotated int parameter whose default is a str."""


def requires_count(count: int = "3") -> int:  # type: ignore[assignment]
    return count


try:
    result = requires_count()
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
