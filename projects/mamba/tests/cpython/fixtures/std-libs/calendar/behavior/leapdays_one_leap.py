# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "leapdays_one_leap"
# subject = "calendar.leapdays"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.leapdays: leapdays(2012, 2013) counts the single leap year 2012"""
import calendar

assert calendar.leapdays(2012, 2013) == 1, 'leapdays one leap'
print("leapdays_one_leap OK")
