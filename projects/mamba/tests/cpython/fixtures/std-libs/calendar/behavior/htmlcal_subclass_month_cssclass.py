# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "htmlcal_subclass_month_cssclass"
# subject = "calendar.HTMLCalendar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.HTMLCalendar: an HTMLCalendar subclass overriding cssclass_month rethemes the per-month wrapper class in formatmonth output"""
import calendar

class ThemedHTMLCal(calendar.HTMLCalendar):
    cssclass_month = "text-center month"


cal = ThemedHTMLCal()
assert 'class="text-center month"' in cal.formatmonth(2017, 5), "month class"
print("htmlcal_subclass_month_cssclass OK")
