# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "htmlcal_formatmonthname_withyear"
# subject = "calendar.HTMLCalendar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.HTMLCalendar: HTMLCalendar.formatmonthname(withyear=True) emits the month name and year inside the fixed 'month' CSS class"""
import calendar

hc = calendar.HTMLCalendar()
assert hc.formatmonthname(2004, 1, withyear=True) == (
    '<tr><th colspan="7" class="month">January 2004</th></tr>'
), "formatmonthname withyear"
print("htmlcal_formatmonthname_withyear OK")
