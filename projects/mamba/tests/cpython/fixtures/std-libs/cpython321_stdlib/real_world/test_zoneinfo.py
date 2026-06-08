# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_zoneinfo"
# subject = "cpython321.test_zoneinfo"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_zoneinfo.py"
# status = "filled"
# ///
"""cpython321.test_zoneinfo: execute CPython 3.12 seed test_zoneinfo"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# test_zoneinfo.py — #3393 axis-1 stdlib zoneinfo AssertionPass seed.
#
# Mamba-authored seed exercising the `zoneinfo` module surface called
# out in the issue:
#   ZoneInfo('UTC'), conversion, .key, available_timezones.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + public surface (hasattr).
#   2. ZoneInfo('UTC') construction + .key + utcoffset(now) is zero.
#   3. ZoneInfo('America/New_York') / ZoneInfo('Asia/Taipei') —
#      construction, .key, non-zero / non-equal offsets vs UTC.
#   4. tzinfo-aware datetime constructed with ZoneInfo carries the
#      ZoneInfo on .tzinfo.
#   5. .astimezone() converts an aware datetime between zones (UTC <->
#      America/New_York at a fixed pin date).
#   6. available_timezones() — non-empty set + 'UTC' membership +
#      'America/New_York' membership.
#   7. ZoneInfoNotFoundError raised on bogus zone name.
#
# Pin date is winter (EST = UTC-5) to keep the offset arithmetic stable
# regardless of host TZDATA version (DST rules vary by year).
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: zoneinfo N asserts` to stdout.

import zoneinfo
import datetime

_ledger: list[int] = []

# 1. Module identity + public surface.
assert zoneinfo.__name__ == "zoneinfo", "zoneinfo.__name__"
_ledger.append(1)
assert hasattr(zoneinfo, "ZoneInfo"), "exposes ZoneInfo"
_ledger.append(1)
assert hasattr(zoneinfo, "available_timezones"), "exposes available_timezones"
_ledger.append(1)
assert hasattr(zoneinfo, "ZoneInfoNotFoundError"), "exposes ZoneInfoNotFoundError"
_ledger.append(1)

# 2. ZoneInfo('UTC').
_utc = zoneinfo.ZoneInfo("UTC")
assert isinstance(_utc, zoneinfo.ZoneInfo), "ZoneInfo() returns a ZoneInfo instance"
_ledger.append(1)
assert _utc.key == "UTC", "ZoneInfo('UTC').key == 'UTC'"
_ledger.append(1)
# utcoffset against any aware-or-naive datetime is timedelta(0) for UTC.
_now_naive = datetime.datetime(2026, 1, 15, 12, 0, 0)
assert _utc.utcoffset(_now_naive) == datetime.timedelta(0), (
    "UTC utcoffset is timedelta(0)"
)
_ledger.append(1)

# 3. Non-UTC zones — America/New_York + Asia/Taipei.
_nyc = zoneinfo.ZoneInfo("America/New_York")
assert _nyc.key == "America/New_York", "ZoneInfo('America/New_York').key"
_ledger.append(1)
_tpe = zoneinfo.ZoneInfo("Asia/Taipei")
assert _tpe.key == "Asia/Taipei", "ZoneInfo('Asia/Taipei').key"
_ledger.append(1)

# Pin to a winter date (EST = UTC-5, Taipei = UTC+8) — DST-stable.
_pin = datetime.datetime(2026, 1, 15, 12, 0, 0)
_nyc_off = _nyc.utcoffset(_pin)
_tpe_off = _tpe.utcoffset(_pin)
assert _nyc_off == datetime.timedelta(hours=-5), (
    "America/New_York utcoffset in January 2026 == -5h"
)
_ledger.append(1)
assert _tpe_off == datetime.timedelta(hours=8), (
    "Asia/Taipei utcoffset in January 2026 == +8h"
)
_ledger.append(1)
assert _nyc_off != _tpe_off, "different zones report different offsets"
_ledger.append(1)
assert _nyc_off != _utc.utcoffset(_pin), "NYC offset != UTC offset"
_ledger.append(1)

# 4. tzinfo-aware datetime carries the ZoneInfo.
_aware_utc = datetime.datetime(2026, 1, 15, 12, 0, 0, tzinfo=_utc)
assert _aware_utc.tzinfo is _utc, "aware datetime carries the ZoneInfo on .tzinfo"
_ledger.append(1)
assert _aware_utc.utcoffset() == datetime.timedelta(0), (
    "aware-UTC datetime .utcoffset() is zero"
)
_ledger.append(1)
# tzname must be a non-empty string.
_tzn = _aware_utc.tzname()
assert isinstance(_tzn, str), "ZoneInfo aware datetime tzname() is a string"
_ledger.append(1)
assert len(_tzn) > 0, "tzname() is non-empty"
_ledger.append(1)

# 5. astimezone — convert UTC to NYC (winter pin).
_in_utc = datetime.datetime(2026, 1, 15, 17, 0, 0, tzinfo=_utc)  # 17:00 UTC
_in_nyc = _in_utc.astimezone(_nyc)
# 17:00 UTC -> 12:00 EST.
assert _in_nyc.hour == 12, "17:00 UTC -> 12:00 EST"
_ledger.append(1)
_nyc_tz = _in_nyc.tzinfo
assert isinstance(_nyc_tz, zoneinfo.ZoneInfo), "astimezone tzinfo is a ZoneInfo"
_ledger.append(1)
assert _nyc_tz.key == "America/New_York", "astimezone keeps target zone"
_ledger.append(1)
# Reverse: convert back to UTC must yield the original moment-in-time.
_back = _in_nyc.astimezone(_utc)
assert _back == _in_utc, "round-trip astimezone preserves instant"
_ledger.append(1)

# 6. available_timezones — populated set, has UTC and well-known zones.
_avail = zoneinfo.available_timezones()
assert isinstance(_avail, set), "available_timezones() returns a set"
_ledger.append(1)
assert len(_avail) > 0, "available_timezones() is non-empty"
_ledger.append(1)
assert "UTC" in _avail, "available_timezones() includes 'UTC'"
_ledger.append(1)
assert "America/New_York" in _avail, "available_timezones() includes 'America/New_York'"
_ledger.append(1)
assert "Asia/Taipei" in _avail, "available_timezones() includes 'Asia/Taipei'"
_ledger.append(1)

# 7. ZoneInfoNotFoundError on bogus zone.
_raised = False
try:
    zoneinfo.ZoneInfo("Not/A/Real/Zone/Surely")
except zoneinfo.ZoneInfoNotFoundError:
    _raised = True
assert _raised == True, "bogus zone name raises ZoneInfoNotFoundError"
_ledger.append(1)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: zoneinfo {len(_ledger)} asserts")
