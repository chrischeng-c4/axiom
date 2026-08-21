# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "leapdays_several"
# subject = "calendar.leapdays"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.leapdays: leapdays(1997, 2020) counts the 5 leap years in the span"""
import calendar

assert calendar.leapdays(1997, 2020) == 5, 'leapdays several'
print("leapdays_several OK")
