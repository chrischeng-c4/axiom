# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "htmlcal_subclass_week_cell_cssclass"
# subject = "calendar.HTMLCalendar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.HTMLCalendar: an HTMLCalendar subclass appending ' text-nowrap' to cssclasses carries that suffix on each populated week-day cell"""
import calendar

class ThemedHTMLCal(calendar.HTMLCalendar):
    cssclasses = [c + " text-nowrap" for c in calendar.HTMLCalendar.cssclasses]


cal = ThemedHTMLCal()
weeks = cal.monthdays2calendar(2017, 5)
assert 'class="wed text-nowrap"' in cal.formatweek(weeks[0]), "week cell class"
print("htmlcal_subclass_week_cell_cssclass OK")
