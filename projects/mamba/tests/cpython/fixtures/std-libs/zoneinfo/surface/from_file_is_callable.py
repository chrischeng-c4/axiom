# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "surface"
# case = "from_file_is_callable"
# subject = "zoneinfo.ZoneInfo.from_file"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zoneinfo.ZoneInfo.from_file: from_file_is_callable (surface)."""
import zoneinfo

assert callable(zoneinfo.ZoneInfo.from_file)
print("from_file_is_callable OK")
