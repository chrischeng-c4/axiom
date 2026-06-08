# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "time_isoformat_roundtrip"
# subject = "datetime.time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.time: time(13,45,30,250000).isoformat() == '13:45:30.250000' and time.fromisoformat round-trips it"""
import datetime

t = datetime.time(13, 45, 30, 250000)
assert t.isoformat() == "13:45:30.250000", f"time iso = {t.isoformat()!r}"
assert datetime.time.fromisoformat(t.isoformat()) == t, "time iso round-trip"
print("time_isoformat_roundtrip OK")
