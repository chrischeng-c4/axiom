# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "itermonthdays_feb2001_fw3_exact"
# subject = "calendar.Calendar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.Calendar: Feb 2001 with firstweekday=3 lines up so itermonthdays yields exactly the 28 real day numbers with no padding"""
import calendar

days = list(calendar.Calendar(firstweekday=3).itermonthdays(2001, 2))
assert days == list(range(1, 29)), ("itermonthdays feb2001", days)
print("itermonthdays_feb2001_fw3_exact OK")
