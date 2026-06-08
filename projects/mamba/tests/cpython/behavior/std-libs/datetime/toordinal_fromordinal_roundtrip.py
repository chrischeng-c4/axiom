# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "toordinal_fromordinal_roundtrip"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.datetime: toordinal/fromordinal are exact inverses at known anchors (1-01-01=1, 1-12-31=365, 2-01-01=366, 1945-11-12=710347) with time-zero on the way back"""
import datetime

for y, m, d, n in [(1, 1, 1, 1), (1, 12, 31, 365), (2, 1, 1, 366),
                   (1945, 11, 12, 710347)]:
    dt = datetime.datetime(y, m, d)
    assert dt.toordinal() == n, f"toordinal({y}) = {dt.toordinal()!r}"
    back = datetime.datetime.fromordinal(n)
    assert back == dt, f"fromordinal({n}) = {back!r}"
    assert back.hour == 0 and back.microsecond == 0, "fromordinal time-zero"
print("toordinal_fromordinal_roundtrip OK")
