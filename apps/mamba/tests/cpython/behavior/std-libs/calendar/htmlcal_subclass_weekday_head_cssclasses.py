# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "htmlcal_subclass_weekday_head_cssclasses"
# subject = "calendar.HTMLCalendar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.HTMLCalendar: an HTMLCalendar subclass overriding cssclasses_weekday_head applies a distinct per-weekday class to each header cell"""
import calendar

class ThemedHTMLCal(calendar.HTMLCalendar):
    cssclasses_weekday_head = ["red", "blue", "green", "lilac",
                               "yellow", "orange", "pink"]


cal = ThemedHTMLCal()
header = cal.formatweekheader()
for color in cal.cssclasses_weekday_head:
    assert ('<th class="%s">' % color) in header, ("weekday head", color)
print("htmlcal_subclass_weekday_head_cssclasses OK")
