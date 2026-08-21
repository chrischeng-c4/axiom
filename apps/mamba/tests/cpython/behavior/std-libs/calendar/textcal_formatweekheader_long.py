# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "textcal_formatweekheader_long"
# subject = "calendar.TextCalendar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.TextCalendar: TextCalendar.formatweekheader(9) emits the wide full-name weekday header row"""
import calendar

tc = calendar.TextCalendar()
assert tc.formatweekheader(9) == (
    "  Monday   Tuesday  Wednesday  Thursday   Friday   Saturday   Sunday "
), "formatweekheader(9)"
print("textcal_formatweekheader_long OK")
