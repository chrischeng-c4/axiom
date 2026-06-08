# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "surface"
# case = "import_zoneinfo"
# subject = "zoneinfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zoneinfo: import_zoneinfo (surface)."""
import zoneinfo

assert hasattr(zoneinfo, "ZoneInfo")
print("import_zoneinfo OK")
