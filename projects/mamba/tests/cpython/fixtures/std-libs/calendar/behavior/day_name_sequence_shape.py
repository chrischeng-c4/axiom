# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "day_name_sequence_shape"
# subject = "calendar.day_name"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.day_name: day_name is a 7-element sequence, all entries distinct, slices and reverses like a sequence"""
import calendar

value = calendar.day_name
assert len(value) == 7, ("day_name", "len")
assert len(value[:]) == 7, ("day_name", "slice len")
assert len(set(value)) == 7, ("day_name", "all distinct")
assert value[::-1] == list(reversed(value)), ("day_name", "reversed")
print("day_name_sequence_shape OK")
