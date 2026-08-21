# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "textcal_prmonth_matches_formatmonth"
# subject = "calendar.TextCalendar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.TextCalendar: TextCalendar.prmonth prints exactly what formatmonth returns (captured via a self-contained stdout redirect)"""
import calendar
import io
import contextlib

result_2004_01 = (
    "    January 2004\n"
    "Mo Tu We Th Fr Sa Su\n"
    "          1  2  3  4\n"
    " 5  6  7  8  9 10 11\n"
    "12 13 14 15 16 17 18\n"
    "19 20 21 22 23 24 25\n"
    "26 27 28 29 30 31\n"
)

tc = calendar.TextCalendar()
buf = io.StringIO()
with contextlib.redirect_stdout(buf):
    tc.prmonth(2004, 1)
assert buf.getvalue() == result_2004_01, "prmonth == formatmonth"
print("textcal_prmonth_matches_formatmonth OK")
