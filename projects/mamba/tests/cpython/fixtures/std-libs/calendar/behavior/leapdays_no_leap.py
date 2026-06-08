# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "leapdays_no_leap"
# subject = "calendar.leapdays"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.leapdays: leapdays(2010, 2011) over a non-leap span counts zero"""
import calendar

assert calendar.leapdays(2010, 2011) == 0, 'leapdays no leap'
print("leapdays_no_leap OK")
