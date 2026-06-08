# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "monthrange_jan_2004"
# subject = "calendar.monthrange"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.monthrange: monthrange returns (weekday_of_first, days_in_month); January 2004 is (3, 31)"""
import calendar

assert calendar.monthrange(2004, 1) == (3, 31), 'monthrange Jan'
print("monthrange_jan_2004 OK")
