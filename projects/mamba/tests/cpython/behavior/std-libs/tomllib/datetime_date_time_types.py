# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "behavior"
# case = "datetime_date_time_types"
# subject = "tomllib.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_misc.py"
# status = "filled"
# ///
"""tomllib.loads: offset datetime parses to datetime.datetime, a bare date to datetime.date, a bare time to datetime.time, with correct year/month fields"""
import tomllib
import datetime

_d = tomllib.loads("""
dt = 2023-01-15T10:30:00Z
date_only = 2023-01-15
time_only = 10:30:00
""")
assert isinstance(_d["dt"], datetime.datetime), f"datetime type = {type(_d['dt'])!r}"
assert isinstance(_d["date_only"], datetime.date), f"date type = {type(_d['date_only'])!r}"
assert isinstance(_d["time_only"], datetime.time), f"time type = {type(_d['time_only'])!r}"
assert _d["dt"].year == 2023, f"year = {_d['dt'].year!r}"
assert _d["dt"].month == 1, f"month = {_d['dt'].month!r}"
assert _d["dt"].day == 15, f"day = {_d['dt'].day!r}"

print("datetime_date_time_types OK")
