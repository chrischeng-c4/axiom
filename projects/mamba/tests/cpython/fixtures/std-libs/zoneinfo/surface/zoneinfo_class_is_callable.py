# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "surface"
# case = "zoneinfo_class_is_callable"
# subject = "zoneinfo.ZoneInfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zoneinfo.ZoneInfo: zoneinfo_class_is_callable (surface)."""
import zoneinfo

assert callable(zoneinfo.ZoneInfo)
print("zoneinfo_class_is_callable OK")
