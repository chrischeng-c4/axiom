# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "yeardatescalendar_yields_dates"
# subject = "calendar.Calendar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.Calendar: yeardatescalendar(2004) yields datetime.date objects in the nested grid; the first real Jan 2004 cell is date(2004, 1, 1)"""
import calendar
import datetime

dates = calendar.Calendar().yeardatescalendar(2004)
assert isinstance(dates[0][0][0][3], datetime.date), "cell is a date"
assert dates[0][0][0][3] == datetime.date(2004, 1, 1), "first real day"
print("yeardatescalendar_yields_dates OK")
