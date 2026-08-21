# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "errors"
# case = "not_found_caught_as_keyerror"
# subject = "zoneinfo.ZoneInfoNotFoundError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo.py"
# status = "filled"
# ///
"""zoneinfo.ZoneInfoNotFoundError: not_found_caught_as_keyerror (errors)."""
import zoneinfo

_raised = False
try:
    zoneinfo.ZoneInfo("No/Such/Timezone")
except KeyError:
    _raised = True
assert _raised, "not_found_caught_as_keyerror: expected KeyError"
print("not_found_caught_as_keyerror OK")
