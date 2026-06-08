# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "behavior"
# case = "invalid_tz_path_is_runtime_warning"
# subject = "zoneinfo.InvalidTZPathWarning"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo.py"
# status = "filled"
# ///
"""zoneinfo.InvalidTZPathWarning: InvalidTZPathWarning is a subclass of RuntimeWarning so callers can filter it as a warning"""
import zoneinfo

assert issubclass(zoneinfo.InvalidTZPathWarning, RuntimeWarning), \
    "InvalidTZPathWarning must subclass RuntimeWarning"
assert issubclass(zoneinfo.InvalidTZPathWarning, Warning), \
    "InvalidTZPathWarning must be a Warning"
# An instance is catchable as a plain RuntimeWarning.
w = zoneinfo.InvalidTZPathWarning("bad path")
assert isinstance(w, RuntimeWarning), type(w).__name__
print("invalid_tz_path_is_runtime_warning OK")
