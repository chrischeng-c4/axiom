# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "day_abbr_sequence_shape"
# subject = "calendar.day_abbr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.day_abbr: day_abbr is a 7-element sequence, all entries distinct, slices and reverses like a sequence"""
import calendar

value = calendar.day_abbr
assert len(value) == 7, ("day_abbr", "len")
assert len(value[:]) == 7, ("day_abbr", "slice len")
assert len(set(value)) == 7, ("day_abbr", "all distinct")
assert value[::-1] == list(reversed(value)), ("day_abbr", "reversed")
print("day_abbr_sequence_shape OK")
