# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "htmlcal_formatmonthname_no_year"
# subject = "calendar.HTMLCalendar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.HTMLCalendar: HTMLCalendar.formatmonthname(withyear=False) emits the month name only, no year"""
import calendar

hc = calendar.HTMLCalendar()
assert hc.formatmonthname(2004, 1, withyear=False) == (
    '<tr><th colspan="7" class="month">January</th></tr>'
), "formatmonthname no year"
print("htmlcal_formatmonthname_no_year OK")
