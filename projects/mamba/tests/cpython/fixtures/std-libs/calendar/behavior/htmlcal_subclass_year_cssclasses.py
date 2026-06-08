# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "htmlcal_subclass_year_cssclasses"
# subject = "calendar.HTMLCalendar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.HTMLCalendar: an HTMLCalendar subclass overriding cssclass_year and cssclass_year_head rethemes the year wrapper and year header in formatyear output"""
import calendar

class ThemedHTMLCal(calendar.HTMLCalendar):
    cssclass_year = "text-italic "
    cssclass_year_head = "lead "


cal = ThemedHTMLCal()
year = cal.formatyear(2017)
assert ('class="%s"' % cal.cssclass_year) in year, "year wrapper class"
assert ('class="%s">%s</th>' % (cal.cssclass_year_head, 2017)) in year, \
    "year head class"
print("htmlcal_subclass_year_cssclasses OK")
