# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "timezone_utc_dst_none"
# subject = "datetime.timezone"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.timezone: timezone.utc.dst(datetime) is always None"""
import datetime

assert datetime.timezone.utc.dst(datetime.datetime(2010, 1, 1)) is None, "utc dst None"
print("timezone_utc_dst_none OK")
