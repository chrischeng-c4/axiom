# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "date_month_13_raises"
# subject = "datetime.date"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date: date_month_13_raises (errors)."""
import datetime

_raised = False
try:
    datetime.date(2024, 13, 1)
except ValueError:
    _raised = True
assert _raised, "date_month_13_raises: expected ValueError"
print("date_month_13_raises OK")
