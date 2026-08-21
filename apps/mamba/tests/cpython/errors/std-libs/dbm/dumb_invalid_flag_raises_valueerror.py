# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "errors"
# case = "dumb_invalid_flag_raises_valueerror"
# subject = "dbm.dumb.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.dumb.open: dumb_invalid_flag_raises_valueerror (errors)."""
import dbm.dumb

_raised = False
try:
    dbm.dumb.open("/tmp/__mamba_dbm_unused__/db", "q")
except ValueError:
    _raised = True
assert _raised, "dumb_invalid_flag_raises_valueerror: expected ValueError"
print("dumb_invalid_flag_raises_valueerror OK")
