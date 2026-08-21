# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "month_abbr_dec"
# subject = "calendar.month_abbr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.month_abbr: month_abbr[12] is the English abbreviation 'Dec'"""
import calendar

assert calendar.month_abbr[12] == "Dec", "month_abbr[12]"
print("month_abbr_dec OK")
