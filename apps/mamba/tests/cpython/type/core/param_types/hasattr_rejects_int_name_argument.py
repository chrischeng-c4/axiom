# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "hasattr_rejects_int_name_argument"
# subject = "hasattr"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""hasattr: hasattr_rejects_int_name_argument (errors)."""
try:
    result = hasattr(1, 2)
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
