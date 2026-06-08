# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "surface"
# case = "reset_tzpath_is_callable"
# subject = "zoneinfo.reset_tzpath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zoneinfo.reset_tzpath: reset_tzpath_is_callable (surface)."""
import zoneinfo

assert callable(zoneinfo.reset_tzpath)
print("reset_tzpath_is_callable OK")
