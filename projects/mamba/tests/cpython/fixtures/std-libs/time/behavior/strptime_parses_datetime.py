# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "strptime_parses_datetime"
# subject = "time.strptime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.strptime: strptime('2023-06-15 12:30:00', '%Y-%m-%d %H:%M:%S') returns a struct_time with year=2023, mon=6, mday=15, hour=12"""
import time

_parsed = time.strptime("2023-06-15 12:30:00", "%Y-%m-%d %H:%M:%S")
assert isinstance(_parsed, time.struct_time), f"strptime type = {type(_parsed)!r}"
assert _parsed.tm_year == 2023, f"parsed year = {_parsed.tm_year!r}"
assert _parsed.tm_mon == 6, f"parsed month = {_parsed.tm_mon!r}"
assert _parsed.tm_mday == 15, f"parsed day = {_parsed.tm_mday!r}"
assert _parsed.tm_hour == 12, f"parsed hour = {_parsed.tm_hour!r}"
print("strptime_parses_datetime OK")
