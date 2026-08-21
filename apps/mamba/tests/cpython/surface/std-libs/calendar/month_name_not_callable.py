# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "month_name_not_callable"
# subject = "calendar.month_name"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.month_name: month_name_not_callable (surface)."""
import calendar

assert not callable(calendar.month_name)
print("month_name_not_callable OK")
