# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "timezone_not_equal_bare_tzinfo"
# subject = "datetime.timezone"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.timezone: a concrete timezone (utc, or a fixed offset) never compares equal to a bare tzinfo() instance"""
import datetime

assert datetime.timezone.utc != datetime.tzinfo(), "utc != bare tzinfo"
assert datetime.timezone(datetime.timedelta(hours=1)) != datetime.tzinfo(), "offset != bare tzinfo"
print("timezone_not_equal_bare_tzinfo OK")
