# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "datetime_field_accessors"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.datetime: datetime(2025,3,15,10,20,30) exposes year/month/day and hour/minute/second; a second instance confirms per-instance fields"""
import datetime

dt = datetime.datetime(2025, 3, 15, 10, 20, 30)
assert (dt.year, dt.month, dt.day) == (2025, 3, 15), f"date part = {dt!r}"
assert (dt.hour, dt.minute, dt.second) == (10, 20, 30), f"time part = {dt!r}"
dt2 = datetime.datetime(1999, 12, 31, 23, 59, 59)
assert (dt2.year, dt2.month, dt2.day) == (1999, 12, 31), f"dt2 date = {dt2!r}"
assert (dt2.hour, dt2.minute, dt2.second) == (23, 59, 59), f"dt2 time = {dt2!r}"
print("datetime_field_accessors OK")
