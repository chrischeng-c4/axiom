# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "isleap_century_not_leap"
# subject = "calendar.isleap"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.isleap: a div-by-100 not-div-by-400 century year (1900) is NOT leap"""
import calendar

assert calendar.isleap(1900) is False, 'isleap(1900)'  # /100 not /400
print("isleap_century_not_leap OK")
