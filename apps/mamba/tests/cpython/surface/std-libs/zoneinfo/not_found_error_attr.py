# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "surface"
# case = "not_found_error_attr"
# subject = "zoneinfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zoneinfo: not_found_error_attr (surface)."""
import zoneinfo

assert hasattr(zoneinfo, "ZoneInfoNotFoundError")
print("not_found_error_attr OK")
