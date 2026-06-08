# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "day_abbr_sun"
# subject = "calendar.day_abbr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.day_abbr: day_abbr[6] is the English abbreviation 'Sun'"""
import calendar

assert calendar.day_abbr[6] == "Sun", "day_abbr[6]"
print("day_abbr_sun OK")
