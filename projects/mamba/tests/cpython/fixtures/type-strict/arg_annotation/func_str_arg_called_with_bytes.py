# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "type-strict"
# lib = "arg_annotation"
# dimension = "errors"
# case = "func_str_arg_called_with_bytes"
# subject = "function positional parameter annotation"
# kind = "semantic"
# xfail = "str argument annotation type enforcement pending; currently MAMBA_TYPE_LEAKED"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-xfail: str argument annotation type enforcement pending; currently MAMBA_TYPE_LEAKED
# mamba-strict-type: TypeError
"""Mamba runtime-type enforcement: str-annotated arg called with bytes.

CPython 3.12: accepts.
Mamba: raises TypeError at call time.
"""


def upper(s: str) -> str:
    return s.upper() if isinstance(s, str) else "<not str>"


try:
    result = upper(b"hi")  # type: ignore[arg-type]
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
