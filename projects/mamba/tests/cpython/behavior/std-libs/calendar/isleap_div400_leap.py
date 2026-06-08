# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "isleap_div400_leap"
# subject = "calendar.isleap"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.isleap: a div-by-400 century year (2000) is a leap year"""
import calendar

assert calendar.isleap(2000) is True, 'isleap(2000)'  # /400 -> leap
print("isleap_div400_leap OK")
