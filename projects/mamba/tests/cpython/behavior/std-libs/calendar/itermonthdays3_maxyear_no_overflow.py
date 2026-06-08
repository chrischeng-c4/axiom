# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "itermonthdays3_maxyear_no_overflow"
# subject = "calendar.Calendar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.Calendar: itermonthdays3 must not overflow at the maximum supported year (datetime.MAXYEAR); it still yields at least the 28 real days"""
import calendar
import datetime

maxed = list(calendar.Calendar().itermonthdays3(datetime.MAXYEAR, 12))
assert len(maxed) >= 28, ("itermonthdays3 maxyear", len(maxed))
print("itermonthdays3_maxyear_no_overflow OK")
