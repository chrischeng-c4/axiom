# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "htmlcal_subclass_month_head_cssclass"
# subject = "calendar.HTMLCalendar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.HTMLCalendar: an HTMLCalendar subclass overriding cssclass_month_head rethemes the month-name header class in formatmonthname output"""
import calendar

class ThemedHTMLCal(calendar.HTMLCalendar):
    cssclass_month_head = "text-center month-head"


cal = ThemedHTMLCal()
assert 'class="text-center month-head"' in cal.formatmonthname(2017, 5), \
    "month-head class"
print("htmlcal_subclass_month_head_cssclass OK")
