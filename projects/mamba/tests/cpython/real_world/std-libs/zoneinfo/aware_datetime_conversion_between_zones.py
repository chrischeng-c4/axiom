# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "real_world"
# case = "aware_datetime_conversion_between_zones"
# subject = "zoneinfo.ZoneInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo.py"
# status = "filled"
# ///
"""zoneinfo.ZoneInfo: a downstream app builds a UTC-aware datetime and converts it to America/New_York and Asia/Tokyo, asserting the localized wall-clock times"""
import datetime
import zoneinfo

# A scheduling app stores an instant in UTC, then renders it per user zone.
src = datetime.datetime(2020, 1, 1, 0, 0, tzinfo=zoneinfo.ZoneInfo("UTC"))

# New York is UTC-5 (EST) in January -> the previous evening, 19:00.
ny = src.astimezone(zoneinfo.ZoneInfo("America/New_York"))
assert (ny.year, ny.month, ny.day, ny.hour, ny.minute) == (2019, 12, 31, 19, 0), \
    (ny.year, ny.month, ny.day, ny.hour, ny.minute)
assert ny.tzname() == "EST", ny.tzname()
assert ny.utcoffset() == datetime.timedelta(hours=-5), ny.utcoffset()

# Tokyo is UTC+9 (JST, no DST) -> the same day, 09:00.
tk = src.astimezone(zoneinfo.ZoneInfo("Asia/Tokyo"))
assert (tk.year, tk.month, tk.day, tk.hour, tk.minute) == (2020, 1, 1, 9, 0), \
    (tk.year, tk.month, tk.day, tk.hour, tk.minute)
assert tk.tzname() == "JST", tk.tzname()
assert tk.utcoffset() == datetime.timedelta(hours=9), tk.utcoffset()

# All three represent the same physical instant.
assert ny.astimezone(datetime.timezone.utc) == tk.astimezone(datetime.timezone.utc) == src
print("aware_datetime_conversion_between_zones OK")
