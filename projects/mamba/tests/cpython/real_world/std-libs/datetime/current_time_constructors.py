# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "real_world"
# case = "current_time_constructors"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.datetime: the present-time entry points work without asserting an exact clock value: now()/today() return instances anchored in the present (year >= 2024), utcnow() (deprecated) still returns a datetime, and combine() rebuilds a datetime from a date and a time"""
import datetime

import warnings

# now() / today() return instances anchored in the present.
now = datetime.datetime.now()
assert isinstance(now, datetime.datetime), f"now type = {type(now)!r}"
assert now.year >= 2024, f"now year = {now.year!r}"

today = datetime.date.today()
assert isinstance(today, datetime.date), f"today type = {type(today)!r}"
assert today.year >= 2024, f"today year = {today.year!r}"

# utcnow() (deprecated in 3.12) still returns a datetime.
with warnings.catch_warnings():
    warnings.simplefilter("ignore")
    utc_now = datetime.datetime.utcnow()
assert isinstance(utc_now, datetime.datetime), f"utcnow type = {type(utc_now)!r}"

# combine() rebuilds a datetime from a separate date and time.
d = datetime.date(2025, 3, 15)
t = datetime.time(10, 30, 45)
combined = datetime.datetime.combine(d, t)
assert combined == datetime.datetime(2025, 3, 15, 10, 30, 45), f"combine = {combined!r}"
assert combined.date() == d, "combined.date()"
assert combined.time() == t, "combined.time()"
print("current_time_constructors OK")
