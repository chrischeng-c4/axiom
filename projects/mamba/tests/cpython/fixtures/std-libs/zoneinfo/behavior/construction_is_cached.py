# /// script
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
