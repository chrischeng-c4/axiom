# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "time_isoformat_micros_padding"
# subject = "datetime.time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.time: time.isoformat() pads microseconds to six digits, equals str(time), and midnight prints '00:00:00'"""
import datetime

assert datetime.time(4, 5, 1, 123).isoformat() == "04:05:01.000123", "micros padded"
assert str(datetime.time(microsecond=10)) == "00:00:00.000010", "str == isoformat"
assert datetime.time().isoformat() == "00:00:00", "midnight isoformat"
print("time_isoformat_micros_padding OK")
