# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "type-strict"
# lib = "var_annotation"
# dimension = "errors"
# case = "int_var_assigned_str"
# subject = "variable annotation assignment"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba runtime-type enforcement: int-annotated var bound to str.

CPython 3.12: annotation is documentation, assignment succeeds.
Mamba: raises TypeError at assignment time.
"""

try:
    x: int = "abc"  # type: ignore[assignment]
    print("no_typeerror:", repr(x))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
