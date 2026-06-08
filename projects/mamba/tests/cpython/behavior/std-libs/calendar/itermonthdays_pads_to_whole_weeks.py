# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "itermonthdays_pads_to_whole_weeks"
# subject = "calendar.Calendar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.Calendar: itermonthdays pads each month to a whole number of weeks (35 or 42 cells), across all firstweekday values and the year boundaries"""
import calendar

for fw in range(7):
    cal = calendar.Calendar(fw)
    for y, m in [(1, 1), (9999, 12)]:
        days = list(cal.itermonthdays(y, m))
        assert len(days) in (35, 42), ("itermonthdays len", fw, y, m, len(days))
print("itermonthdays_pads_to_whole_weeks OK")
