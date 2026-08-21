# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "isleap_common_not_leap"
# subject = "calendar.isleap"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.isleap: a non-div-by-4 year (2023) is not a leap year"""
import calendar

assert calendar.isleap(2023) is False, 'isleap(2023)'
print("isleap_common_not_leap OK")
