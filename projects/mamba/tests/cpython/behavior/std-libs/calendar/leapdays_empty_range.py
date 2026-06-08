# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "leapdays_empty_range"
# subject = "calendar.leapdays"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.leapdays: leapdays(y, y) over an empty range counts zero"""
import calendar

assert calendar.leapdays(2010, 2010) == 0, 'leapdays empty range'
print("leapdays_empty_range OK")
