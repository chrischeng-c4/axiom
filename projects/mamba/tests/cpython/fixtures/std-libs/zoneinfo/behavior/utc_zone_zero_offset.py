# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "behavior"
# case = "utc_zone_zero_offset"
# subject = "zoneinfo.ZoneInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo.py"
# status = "filled"
# ///
"""zoneinfo.ZoneInfo: ZoneInfo('UTC') yields a zero utcoffset, zero dst, and tzname 'UTC' for any datetime"""
import datetime
import zoneinfo

utc = zoneinfo.ZoneInfo("UTC")
dt = datetime.datetime(2020, 1, 1, 12, 0, tzinfo=utc)
assert dt.utcoffset() == datetime.timedelta(0), dt.utcoffset()
assert dt.dst() == datetime.timedelta(0), dt.dst()
assert dt.tzname() == "UTC", dt.tzname()

# Offset is constant across the year (no DST in UTC).
summer = datetime.datetime(2020, 7, 1, 12, 0, tzinfo=utc)
assert summer.utcoffset() == datetime.timedelta(0), summer.utcoffset()
assert summer.tzname() == "UTC", summer.tzname()
print("utc_zone_zero_offset OK")
