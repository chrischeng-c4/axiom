# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "date_day_30_february_raises"
# subject = "datetime.date"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date: date_day_30_february_raises (errors)."""
import datetime

_raised = False
try:
    datetime.date(2024, 2, 30)
except ValueError:
    _raised = True
assert _raised, "date_day_30_february_raises: expected ValueError"
print("date_day_30_february_raises OK")
