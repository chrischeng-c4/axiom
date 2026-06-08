# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "surface"
# case = "clear_cache_is_callable"
# subject = "zoneinfo.ZoneInfo.clear_cache"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zoneinfo.ZoneInfo.clear_cache: clear_cache_is_callable (surface)."""
import zoneinfo

assert callable(zoneinfo.ZoneInfo.clear_cache)
print("clear_cache_is_callable OK")
