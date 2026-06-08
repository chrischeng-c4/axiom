# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "leapdays_upper_exclusive"
# subject = "calendar.leapdays"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.leapdays: leapdays is half-open; the leap year 2012 at the upper bound is excluded so leapdays(2010, 2012) == 0"""
import calendar

assert calendar.leapdays(2010, 2012) == 0, 'leapdays upper exclusive'
print("leapdays_upper_exclusive OK")
