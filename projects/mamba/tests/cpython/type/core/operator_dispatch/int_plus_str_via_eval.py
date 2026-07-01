# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "operator_dispatch"
# dimension = "type"
# case = "int_plus_str_via_eval"
# subject = "operator dispatch TypeError"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba runtime-type enforcement: cross-type `+` via eval().

CPython 3.12: `1 + 'a'` raises TypeError at evaluation. Mamba's
strict typing also rejects, but the rejection routes through a
runtime TypeError when fed through `eval()` (the only way to defer
parsing past compile-time typing).
"""

try:
    result = eval("1 + 'a'")
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
