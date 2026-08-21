# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "itermonthdays4_yields_ymd_weekday_tuples"
# subject = "calendar.Calendar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.Calendar: itermonthdays4 yields (year, month, day, weekday) 4-tuples; check first and last for Feb 2001 with firstweekday=3"""
import calendar

days4 = list(calendar.Calendar(firstweekday=3).itermonthdays4(2001, 2))
assert days4[0] == (2001, 2, 1, 3), ("itermonthdays4 first", days4[0])
assert days4[-1] == (2001, 2, 28, 2), ("itermonthdays4 last", days4[-1])
print("itermonthdays4_yields_ymd_weekday_tuples OK")
