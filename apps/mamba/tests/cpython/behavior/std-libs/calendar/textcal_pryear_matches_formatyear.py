# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "textcal_pryear_matches_formatyear"
# subject = "calendar.TextCalendar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.TextCalendar: TextCalendar.pryear prints exactly what formatyear returns (captured via stdout redirect)"""
import calendar
import io
import contextlib

tc = calendar.TextCalendar()
year = tc.formatyear(2004)
buf = io.StringIO()
with contextlib.redirect_stdout(buf):
    tc.pryear(2004)
assert buf.getvalue() == year, "pryear == formatyear"
print("textcal_pryear_matches_formatyear OK")
