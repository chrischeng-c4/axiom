# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "behavior"
# case = "available_timezones_returns_set"
# subject = "zoneinfo.available_timezones"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo.py"
# status = "filled"
# ///
"""zoneinfo.available_timezones: available_timezones() returns a non-empty set of str keys that includes the well-known UTC zone"""
import zoneinfo

tzs = zoneinfo.available_timezones()
assert isinstance(tzs, set), type(tzs).__name__
# Every entry is a string key (sample-check a bounded slice for determinism).
for key in list(tzs)[:50]:
    assert isinstance(key, str), key
# On any standard tzdata install the set is non-empty and includes UTC; the
# CPython oracle on this platform ships full tzdata, so assert it directly.
assert tzs, "available_timezones() should be non-empty on a tzdata install"
assert "UTC" in tzs, "UTC should be among the available zones"
print("available_timezones_returns_set OK")
