# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "aware_datetime_isoformat_offset"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.datetime: a datetime built with tzinfo=timezone.utc keeps that tzinfo identity and renders '+00:00' in isoformat()"""
import datetime

utc_dt = datetime.datetime(2023, 6, 15, 12, 0, 0, tzinfo=datetime.timezone.utc)
assert utc_dt.tzinfo is datetime.timezone.utc, "tzinfo set"
tz_str = utc_dt.isoformat()
assert "+00:00" in tz_str, f"UTC isoformat = {tz_str!r}"
print("aware_datetime_isoformat_offset OK")
