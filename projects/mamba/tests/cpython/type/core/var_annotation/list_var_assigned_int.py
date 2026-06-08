# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "var_annotation"
# dimension = "type"
# case = "list_var_assigned_int"
# subject = "variable annotation assignment"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba runtime-type enforcement: list-annotated var bound to int.

CPython 3.12: assignment succeeds.
Mamba: raises TypeError at assignment time.
"""

try:
    xs: list = 7  # type: ignore[assignment]
    print("no_typeerror:", repr(xs))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
