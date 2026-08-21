# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "timezone_str_is_name"
# subject = "datetime.timezone"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.timezone: str(timezone) equals its tzname(None); a named offset reports its given name"""
import datetime

ACDT = datetime.timezone(datetime.timedelta(hours=9.5), "ACDT")
EST = datetime.timezone(-datetime.timedelta(hours=5), "EST")
for tz in (ACDT, EST, datetime.timezone.utc,
           datetime.timezone.min, datetime.timezone.max):
    assert str(tz) == tz.tzname(None), f"str = {str(tz)!r}"
assert ACDT.tzname(None) == "ACDT", f"ACDT name = {ACDT.tzname(None)!r}"
print("timezone_str_is_name OK")
