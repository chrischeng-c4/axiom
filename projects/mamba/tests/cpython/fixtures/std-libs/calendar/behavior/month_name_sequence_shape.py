# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "month_name_sequence_shape"
# subject = "calendar.month_name"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.month_name: month_name is a 13-element sequence (index 0 is the empty string), all entries distinct, slices and reverses like a sequence"""
import calendar

value = calendar.month_name
assert len(value) == 13, ("month_name", "len")
assert len(value[:]) == 13, ("month_name", "slice len")
assert value[0] == "", ("month_name", "index 0 empty")
assert len(set(value)) == 13, ("month_name", "all distinct")
assert value[::-1] == list(reversed(value)), ("month_name", "reversed")
print("month_name_sequence_shape OK")
