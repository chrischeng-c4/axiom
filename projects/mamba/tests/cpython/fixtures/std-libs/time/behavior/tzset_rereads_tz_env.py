# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "tzset_rereads_tz_env"
# subject = "time.tzset"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.tzset: tzset re-reads the TZ env var: UTC+0 makes gmtime==localtime with daylight/timezone 0 and tm_isdst 0; EST+05EDT makes gmtime!=localtime with tzname ('EST','EDT'), daylight 1, timezone 18000, altzone 14400, December tm_isdst 0; original TZ restored in finally"""
import os
import time

if not hasattr(time, "tzset"):
    # tzset is Unix-only; nothing to assert on platforms without it.
    print("tzset_rereads_tz_env OK (skipped: not available)")
    raise SystemExit(0)

# 2002-12-25 00:00:00 UTC — a fixed instant for reproducible checks.
XMAS_2002 = 1040774400.0
EASTERN = "EST+05EDT,M4.1.0,M10.5.0"
UTC = "UTC+0"

_saved_tz = os.environ.get("TZ")
try:
    # UTC: local time equals UTC, no DST, zero offset.
    os.environ["TZ"] = UTC
    time.tzset()
    assert time.gmtime(XMAS_2002) == time.localtime(XMAS_2002), "UTC: gmtime == localtime"
    assert time.daylight == 0, f"UTC daylight = {time.daylight!r}"
    assert time.timezone == 0, f"UTC timezone = {time.timezone!r}"
    assert time.localtime(XMAS_2002).tm_isdst == 0, "UTC: not in DST"

    # US Eastern: local time diverges from UTC, DST rules active.
    os.environ["TZ"] = EASTERN
    time.tzset()
    assert time.gmtime(XMAS_2002) != time.localtime(XMAS_2002), "EST: gmtime != localtime"
    assert time.tzname == ("EST", "EDT"), f"EST tzname = {time.tzname!r}"
    assert len(time.tzname) == 2, "tzname has 2 entries"
    assert time.daylight == 1, f"EST daylight = {time.daylight!r}"
    assert time.timezone == 18000, f"EST timezone = {time.timezone!r}"  # +5h
    assert time.altzone == 14400, f"EST altzone = {time.altzone!r}"     # +4h (DST)
    # December is standard time, not DST, in the northern hemisphere.
    assert time.localtime(XMAS_2002).tm_isdst == 0, "EST: December not in DST"
finally:
    if _saved_tz is not None:
        os.environ["TZ"] = _saved_tz
    elif "TZ" in os.environ:
        del os.environ["TZ"]
    time.tzset()
print("tzset_rereads_tz_env OK")
