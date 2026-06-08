# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "import_calendar"
# subject = "calendar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar: import_calendar (surface)."""
import calendar

assert hasattr(calendar, "isleap")
print("import_calendar OK")
