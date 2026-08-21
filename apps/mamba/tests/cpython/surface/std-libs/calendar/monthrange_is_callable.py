# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "monthrange_is_callable"
# subject = "calendar.monthrange"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.monthrange: monthrange_is_callable (surface)."""
import calendar

assert callable(calendar.monthrange)
print("monthrange_is_callable OK")
