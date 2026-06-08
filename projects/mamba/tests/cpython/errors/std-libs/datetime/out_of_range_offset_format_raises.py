# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "out_of_range_offset_format_raises"
# subject = "datetime.tzinfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.tzinfo: a custom tzinfo whose utcoffset is +/-1439 minutes formats fine, but a 1440-minute offset is rejected with ValueError at format time"""
import datetime

class Edgy(datetime.tzinfo):
    def __init__(self, minutes):
        self.offset = datetime.timedelta(minutes=minutes)
    def utcoffset(self, dt):
        return self.offset

# +/-1439 minutes is the legal boundary; it formats fine.
ok = datetime.time(1, 2, 3, tzinfo=Edgy(1439))
assert str(ok) == "01:02:03+23:59", f"edge offset str = {str(ok)!r}"

# 1440 minutes is out of range and is rejected at format time.
_raised = False
try:
    str(datetime.time(1, 2, 3, tzinfo=Edgy(1440)))
except ValueError:
    _raised = True
assert _raised, "offset_1440: expected ValueError"
print("out_of_range_offset_format_raises OK")
