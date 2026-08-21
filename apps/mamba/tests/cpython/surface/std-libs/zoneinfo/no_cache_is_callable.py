# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "surface"
# case = "no_cache_is_callable"
# subject = "zoneinfo.ZoneInfo.no_cache"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zoneinfo.ZoneInfo.no_cache: no_cache_is_callable (surface)."""
import zoneinfo

assert callable(zoneinfo.ZoneInfo.no_cache)
print("no_cache_is_callable OK")
