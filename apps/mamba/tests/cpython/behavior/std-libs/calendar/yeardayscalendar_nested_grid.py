# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "yeardayscalendar_nested_grid"
# subject = "calendar.Calendar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.Calendar: yeardayscalendar(2004) nests 4 three-month rows, 3 months each, weeks of 7 day numbers (0 = padding); Jan 2004 first week is [0, 0, 0, 1, 2, 3, 4]"""
import calendar

grid = calendar.Calendar().yeardayscalendar(2004)
assert len(grid) == 4, "4 three-month rows"
assert all(len(row) == 3 for row in grid), "3 months per row"
jan = grid[0][0]
assert all(len(week) == 7 for week in jan), "7 cells per week"
assert jan[0] == [0, 0, 0, 1, 2, 3, 4], ("jan week0", jan[0])
print("yeardayscalendar_nested_grid OK")
