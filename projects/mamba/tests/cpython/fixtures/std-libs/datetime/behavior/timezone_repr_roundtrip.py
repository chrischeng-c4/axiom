# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "timezone_repr_roundtrip"
# subject = "datetime.timezone"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.timezone: repr(timezone) is a valid constructor expression: eval(repr(tz)) == tz for named offsets and utc/min/max"""
import datetime

timezone = datetime.timezone  # eval'd reprs reference datetime.timezone(...)
ACDT = datetime.timezone(datetime.timedelta(hours=9.5), "ACDT")
EST = datetime.timezone(-datetime.timedelta(hours=5), "EST")
for tz in (ACDT, EST, datetime.timezone.utc,
           datetime.timezone.min, datetime.timezone.max):
    assert tz == eval(repr(tz)), f"repr round-trip = {repr(tz)!r}"
print("timezone_repr_roundtrip OK")
