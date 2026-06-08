# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "isleap_div4_leap"
# subject = "calendar.isleap"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.isleap: a plain div-by-4 year (2024) is a leap year"""
import calendar

assert calendar.isleap(2024) is True, 'isleap(2024)'
print("isleap_div4_leap OK")
