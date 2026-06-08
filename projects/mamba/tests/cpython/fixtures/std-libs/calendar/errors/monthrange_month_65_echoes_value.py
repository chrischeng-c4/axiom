# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "errors"
# case = "monthrange_month_65_echoes_value"
# subject = "calendar.monthrange"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.monthrange: monthrange(2004, 65) raises IllegalMonthError whose message text echoes the offending month number 65"""
import calendar

try:
    calendar.monthrange(2004, 65)
    print("month65: no_raise")
except calendar.IllegalMonthError as e:
    print("month65:", type(e).__name__, "echoes 65:", "65" in str(e))
print("monthrange_month_65_echoes_value OK")
