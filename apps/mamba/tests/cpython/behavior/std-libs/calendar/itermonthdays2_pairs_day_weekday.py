# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "itermonthdays2_pairs_day_weekday"
# subject = "calendar.Calendar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.Calendar: itermonthdays2 pairs each day with its weekday; the run starts at firstweekday and ends one before it (mod 7)"""
import calendar

for fw in range(7):
    pairs = list(calendar.Calendar(fw).itermonthdays2(1, 1))
    assert pairs[0][1] == fw, ("first weekday", fw, pairs[0])
    assert pairs[-1][1] == (fw - 1) % 7, ("last weekday", fw, pairs[-1])
print("itermonthdays2_pairs_day_weekday OK")
