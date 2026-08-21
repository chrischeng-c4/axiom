# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "surface"
# case = "tzpath_attr"
# subject = "zoneinfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zoneinfo: tzpath_attr (surface)."""
import zoneinfo

assert hasattr(zoneinfo, "TZPATH")
print("tzpath_attr OK")
