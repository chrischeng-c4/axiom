# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "errors"
# case = "unknown_zone_raises_not_found"
# subject = "zoneinfo.ZoneInfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo.py"
# status = "filled"
# ///
"""zoneinfo.ZoneInfo: unknown_zone_raises_not_found (errors)."""
import zoneinfo

_raised = False
try:
    zoneinfo.ZoneInfo("No/Such/Timezone")
except zoneinfo.ZoneInfoNotFoundError:
    _raised = True
assert _raised, "unknown_zone_raises_not_found: expected zoneinfo.ZoneInfoNotFoundError"
print("unknown_zone_raises_not_found OK")
