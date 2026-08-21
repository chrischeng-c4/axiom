# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "errors"
# case = "path_traversal_key_raises_value_error"
# subject = "zoneinfo.ZoneInfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo.py"
# status = "filled"
# ///
"""zoneinfo.ZoneInfo: path_traversal_key_raises_value_error (errors)."""
import zoneinfo

_raised = False
try:
    zoneinfo.ZoneInfo("../etc/passwd")
except ValueError:
    _raised = True
assert _raised, "path_traversal_key_raises_value_error: expected ValueError"
print("path_traversal_key_raises_value_error OK")
