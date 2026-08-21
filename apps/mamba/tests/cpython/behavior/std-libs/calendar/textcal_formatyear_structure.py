# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "textcal_formatyear_structure"
# subject = "calendar.TextCalendar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.TextCalendar: TextCalendar.formatyear is a 12-month grid carrying the year, spanning January..December, with 12 weekday headers"""
import calendar

tc = calendar.TextCalendar()
year = tc.formatyear(2004)
assert "2004" in year, "formatyear contains year"
assert "January" in year and "December" in year, "formatyear spans months"
assert year.count("Mo Tu We Th Fr Sa Su") == 12, "formatyear has 12 headers"
print("textcal_formatyear_structure OK")
