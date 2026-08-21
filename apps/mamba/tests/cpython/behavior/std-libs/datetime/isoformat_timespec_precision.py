# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "isoformat_timespec_precision"
# subject = "datetime.time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.time: isoformat(timespec=...) controls trailing precision (hours/minutes/seconds/milliseconds/microseconds/auto), and auto drops the fraction when microseconds are zero"""
import datetime

full = datetime.time(12, 34, 56, 123456)
assert full.isoformat(timespec="hours") == "12", "timespec hours"
assert full.isoformat(timespec="minutes") == "12:34", "timespec minutes"
assert full.isoformat(timespec="seconds") == "12:34:56", "timespec seconds"
assert full.isoformat(timespec="milliseconds") == "12:34:56.123", "timespec millis"
assert full.isoformat(timespec="microseconds") == "12:34:56.123456", "timespec micros"
assert full.isoformat(timespec="auto") == "12:34:56.123456", "timespec auto"

# auto drops the fractional part when microseconds are zero.
assert datetime.time(12, 34, 56).isoformat(timespec="auto") == "12:34:56", "auto no-frac"
assert datetime.time(12, 34, 56).isoformat(timespec="milliseconds") == "12:34:56.000", "millis zero-pad"
print("isoformat_timespec_precision OK")
