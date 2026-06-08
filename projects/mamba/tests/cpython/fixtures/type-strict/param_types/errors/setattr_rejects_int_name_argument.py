# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "type-strict"
# lib = "param_types"
# dimension = "errors"
# case = "setattr_rejects_int_name_argument"
# subject = "setattr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""setattr: setattr_rejects_int_name_argument (errors)."""
try:
    result = setattr(1, 2, 3)
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
