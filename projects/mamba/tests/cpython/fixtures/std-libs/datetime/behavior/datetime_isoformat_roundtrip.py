# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "datetime_isoformat_roundtrip"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.datetime: datetime.isoformat() yields YYYY-MM-DDTHH:MM:SS, fromisoformat round-trips it, and sep=' ' swaps the date/time separator"""
import datetime

dt = datetime.datetime(2023, 6, 15, 13, 45, 30)
assert dt.isoformat() == "2023-06-15T13:45:30", f"dt iso = {dt.isoformat()!r}"
assert datetime.datetime.fromisoformat(dt.isoformat()) == dt, "dt iso round-trip"
assert dt.isoformat(sep=" ") == "2023-06-15 13:45:30", f"sep iso = {dt.isoformat(sep=' ')!r}"
print("datetime_isoformat_roundtrip OK")
