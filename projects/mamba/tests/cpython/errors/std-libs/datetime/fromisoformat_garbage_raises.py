# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "fromisoformat_garbage_raises"
# subject = "datetime.date.fromisoformat"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date.fromisoformat: fromisoformat_garbage_raises (errors)."""
import datetime

_raised = False
try:
    datetime.date.fromisoformat("garbage")
except ValueError:
    _raised = True
assert _raised, "fromisoformat_garbage_raises: expected ValueError"
print("fromisoformat_garbage_raises OK")
