# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "surface"
# case = "invalid_tz_path_warning_attr"
# subject = "zoneinfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zoneinfo: invalid_tz_path_warning_attr (surface)."""
import zoneinfo

assert hasattr(zoneinfo, "InvalidTZPathWarning")
print("invalid_tz_path_warning_attr OK")
