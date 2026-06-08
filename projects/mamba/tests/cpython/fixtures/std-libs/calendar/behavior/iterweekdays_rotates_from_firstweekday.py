# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "iterweekdays_rotates_from_firstweekday"
# subject = "calendar.Calendar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.Calendar: Calendar(fw).iterweekdays() yields a rotation of 0..6 starting at firstweekday, for every firstweekday"""
import calendar

week0 = list(range(7))
for fw in range(7):
    week = list(calendar.Calendar(fw).iterweekdays())
    expected = week0[fw:] + week0[:fw]
    assert week == expected, ("iterweekdays", fw, week)
print("iterweekdays_rotates_from_firstweekday OK")
