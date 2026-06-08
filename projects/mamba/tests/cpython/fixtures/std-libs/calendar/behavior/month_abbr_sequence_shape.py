# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "month_abbr_sequence_shape"
# subject = "calendar.month_abbr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.month_abbr: month_abbr is a 13-element sequence (index 0 is the empty string), all entries distinct, slices and reverses like a sequence"""
import calendar

value = calendar.month_abbr
assert len(value) == 13, ("month_abbr", "len")
assert len(value[:]) == 13, ("month_abbr", "slice len")
assert value[0] == "", ("month_abbr", "index 0 empty")
assert len(set(value)) == 13, ("month_abbr", "all distinct")
assert value[::-1] == list(reversed(value)), ("month_abbr", "reversed")
print("month_abbr_sequence_shape OK")
