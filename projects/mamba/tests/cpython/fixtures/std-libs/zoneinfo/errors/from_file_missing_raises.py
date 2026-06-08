# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "errors"
# case = "from_file_missing_raises"
# subject = "zoneinfo.ZoneInfo.from_file"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo.py"
# status = "filled"
# ///
"""zoneinfo.ZoneInfo.from_file: from_file_missing_raises (errors)."""
import zoneinfo

_raised = False
try:
    zoneinfo.ZoneInfo.from_file(open("/no/such/tz/file", "rb"), key="X")
except FileNotFoundError:
    _raised = True
assert _raised, "from_file_missing_raises: expected FileNotFoundError"
print("from_file_missing_raises OK")
