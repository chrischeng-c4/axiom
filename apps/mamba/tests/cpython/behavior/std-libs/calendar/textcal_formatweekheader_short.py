# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "textcal_formatweekheader_short"
# subject = "calendar.TextCalendar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.TextCalendar: TextCalendar.formatweekheader(2) emits the 2-char weekday header row"""
import calendar

tc = calendar.TextCalendar()
assert tc.formatweekheader(2) == "Mo Tu We Th Fr Sa Su", "formatweekheader(2)"
print("textcal_formatweekheader_short OK")
