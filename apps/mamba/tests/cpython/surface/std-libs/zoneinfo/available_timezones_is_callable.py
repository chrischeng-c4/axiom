# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "surface"
# case = "available_timezones_is_callable"
# subject = "zoneinfo.available_timezones"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zoneinfo.available_timezones: available_timezones_is_callable (surface)."""
import zoneinfo

assert callable(zoneinfo.available_timezones)
print("available_timezones_is_callable OK")
