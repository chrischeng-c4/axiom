use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/zoneinfo/available_timezones_returns_set.py`.
#[test]
fn test_gen_behavior_std_libs_zoneinfo_available_timezones_returns_set() {
    let out = jit_capture(r###"# /// script
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
"###);
    assert_output(&out, r###"available_timezones_returns_set OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zoneinfo/construction_is_cached.py`.
#[test]
fn test_gen_behavior_std_libs_zoneinfo_construction_is_cached() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "behavior"
# case = "construction_is_cached"
# subject = "zoneinfo.ZoneInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo.py"
# status = "filled"
# ///
"""zoneinfo.ZoneInfo: repeated ZoneInfo(key) returns the same cached instance; no_cache(key) returns a distinct one"""
import zoneinfo

# Same key constructed twice hits the strong cache -> identical object.
a = zoneinfo.ZoneInfo("UTC")
b = zoneinfo.ZoneInfo("UTC")
assert a is b, "repeated ZoneInfo('UTC') should be cached to one instance"

# no_cache() deliberately bypasses the cache -> a distinct object, same key.
c = zoneinfo.ZoneInfo.no_cache("UTC")
assert c is not a, "no_cache should return a fresh instance"
assert c.key == a.key == "UTC", (c.key, a.key)
print("construction_is_cached OK")
"###);
    assert_output(&out, r###"construction_is_cached OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zoneinfo/dst_transition_offsets.py`.
#[test]
fn test_gen_behavior_std_libs_zoneinfo_dst_transition_offsets() {
    let out = jit_capture(r###"# /// script
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
"###);
    assert_output(&out, r###"dst_transition_offsets OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zoneinfo/import_succeeds.py`.
#[test]
fn test_gen_behavior_std_libs_zoneinfo_import_succeeds() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "behavior"
# case = "import_succeeds"
# subject = "zoneinfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo.py"
# status = "filled"
# ///
"""zoneinfo: the module imports and exposes its public surface (ZoneInfo, available_timezones, TZPATH)"""
import zoneinfo

assert zoneinfo is not None
assert callable(zoneinfo.ZoneInfo), "ZoneInfo should be callable"
assert callable(zoneinfo.available_timezones), "available_timezones should be callable"
assert callable(zoneinfo.reset_tzpath), "reset_tzpath should be callable"
assert hasattr(zoneinfo, "TZPATH"), "module should expose TZPATH"
assert hasattr(zoneinfo, "ZoneInfoNotFoundError"), "module should expose ZoneInfoNotFoundError"
assert hasattr(zoneinfo, "InvalidTZPathWarning"), "module should expose InvalidTZPathWarning"
print("import_succeeds OK")
"###);
    assert_output(&out, r###"import_succeeds OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zoneinfo/invalid_tz_path_is_runtime_warning.py`.
#[test]
fn test_gen_behavior_std_libs_zoneinfo_invalid_tz_path_is_runtime_warning() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "behavior"
# case = "invalid_tz_path_is_runtime_warning"
# subject = "zoneinfo.InvalidTZPathWarning"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo.py"
# status = "filled"
# ///
"""zoneinfo.InvalidTZPathWarning: InvalidTZPathWarning is a subclass of RuntimeWarning so callers can filter it as a warning"""
import zoneinfo

assert issubclass(zoneinfo.InvalidTZPathWarning, RuntimeWarning), \
    "InvalidTZPathWarning must subclass RuntimeWarning"
assert issubclass(zoneinfo.InvalidTZPathWarning, Warning), \
    "InvalidTZPathWarning must be a Warning"
# An instance is catchable as a plain RuntimeWarning.
w = zoneinfo.InvalidTZPathWarning("bad path")
assert isinstance(w, RuntimeWarning), type(w).__name__
print("invalid_tz_path_is_runtime_warning OK")
"###);
    assert_output(&out, r###"invalid_tz_path_is_runtime_warning OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zoneinfo/key_attribute_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_zoneinfo_key_attribute_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "behavior"
# case = "key_attribute_roundtrip"
# subject = "zoneinfo.ZoneInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo.py"
# status = "filled"
# ///
"""zoneinfo.ZoneInfo: a constructed ZoneInfo carries its lookup key on .key and str() returns that key"""
import zoneinfo

for key in ["UTC", "America/New_York"]:
    zi = zoneinfo.ZoneInfo(key)
    assert zi.key == key, (zi.key, key)
    assert str(zi) == key, (str(zi), key)
    assert repr(zi) == "zoneinfo.ZoneInfo(key=%r)" % key, repr(zi)
print("key_attribute_roundtrip OK")
"###);
    assert_output(&out, r###"key_attribute_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zoneinfo/utc_zone_zero_offset.py`.
#[test]
fn test_gen_behavior_std_libs_zoneinfo_utc_zone_zero_offset() {
    let out = jit_capture(r###"# /// script
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
"###);
    assert_output(&out, r###"utc_zone_zero_offset OK
"###);
}
