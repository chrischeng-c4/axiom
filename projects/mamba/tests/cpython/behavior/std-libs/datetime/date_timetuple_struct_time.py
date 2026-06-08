# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "date_timetuple_struct_time"
# subject = "datetime.date"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date: date.timetuple() returns a time.struct_time whose tm_year/tm_mon/tm_mday match the date"""
import datetime

import time as _time

d = datetime.date(2023, 11, 5)
tt = d.timetuple()
assert isinstance(tt, _time.struct_time), f"timetuple type = {type(tt)!r}"
assert tt.tm_year == 2023, f"tt year = {tt.tm_year!r}"
assert tt.tm_mon == 11, f"tt mon = {tt.tm_mon!r}"
assert tt.tm_mday == 5, f"tt mday = {tt.tm_mday!r}"
print("date_timetuple_struct_time OK")
