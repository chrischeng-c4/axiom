# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "int_rejects_str_base_argument"
# subject = "int"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""int: int_rejects_str_base_argument (errors)."""
try:
    result = int("10", "2")
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
