# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "monthrange_feb_leap"
# subject = "calendar.monthrange"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.monthrange: February of leap year 2004 has 29 days -> monthrange (6, 29)"""
import calendar

assert calendar.monthrange(2004, 2) == (6, 29), 'monthrange Feb leap'
print("monthrange_feb_leap OK")
