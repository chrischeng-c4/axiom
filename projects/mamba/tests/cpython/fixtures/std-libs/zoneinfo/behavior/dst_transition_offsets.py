# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "behavior"
# case = "dst_transition_offsets"
# subject = "zoneinfo.ZoneInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo.py"
# status = "filled"
# ///
"""zoneinfo.ZoneInfo: America/New_York reports EST/-5h in January and EDT/-4h with a 1h dst in July"""
import datetime
import zoneinfo

ny = zoneinfo.ZoneInfo("America/New_York")

# Winter: standard time, no DST.
winter = datetime.datetime(2020, 1, 1, 12, 0, tzinfo=ny)
assert winter.utcoffset() == datetime.timedelta(hours=-5), winter.utcoffset()
assert winter.tzname() == "EST", winter.tzname()
assert winter.dst() == datetime.timedelta(0), winter.dst()

# Summer: daylight saving in effect, +1h.
summer = datetime.datetime(2020, 7, 1, 12, 0, tzinfo=ny)
assert summer.utcoffset() == datetime.timedelta(hours=-4), summer.utcoffset()
assert summer.tzname() == "EDT", summer.tzname()
assert summer.dst() == datetime.timedelta(hours=1), summer.dst()
print("dst_transition_offsets OK")
