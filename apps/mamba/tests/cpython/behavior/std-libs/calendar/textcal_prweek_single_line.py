# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "textcal_prweek_single_line"
# subject = "calendar.TextCalendar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.TextCalendar: TextCalendar.prweek prints a single formatted week line with no trailing newline (captured via stdout redirect)"""
import calendar
import io
import contextlib

tc = calendar.TextCalendar()
week = [(1, 0), (2, 1), (3, 2), (4, 3), (5, 4), (6, 5), (7, 6)]
buf = io.StringIO()
with contextlib.redirect_stdout(buf):
    tc.prweek(week, 1)
assert buf.getvalue() == " 1  2  3  4  5  6  7", "prweek line"
print("textcal_prweek_single_line OK")
