# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "localtime_mktime_roundtrip"
# subject = "time.localtime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.localtime: localtime exposes tm_gmtoff and tm_zone; localtime(mktime(localtime(now))) round-trips equal and preserves tm_gmtoff and tm_zone"""
import time

_now = time.time()
_lt = time.localtime(_now)
assert hasattr(_lt, "tm_gmtoff"), "struct_time has tm_gmtoff"
assert hasattr(_lt, "tm_zone"), "struct_time has tm_zone"
_back = time.localtime(time.mktime(_lt))
assert _back == _lt, "localtime(mktime(localtime)) == localtime"
assert _back.tm_gmtoff == _lt.tm_gmtoff, "tm_gmtoff preserved"
assert _back.tm_zone == _lt.tm_zone, "tm_zone preserved"
print("localtime_mktime_roundtrip OK")
