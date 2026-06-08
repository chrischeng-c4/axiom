# /// script
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
