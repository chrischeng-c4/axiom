# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "date_difference_is_timedelta"
# subject = "datetime.date"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date: subtracting two dates yields a timedelta; Dec31 2023 - Jan1 2023 is 364 days"""
import datetime

diff = datetime.date(2023, 12, 31) - datetime.date(2023, 1, 1)
assert isinstance(diff, datetime.timedelta), f"diff type = {type(diff)!r}"
assert diff.days == 364, f"diff days = {diff.days!r}"
print("date_difference_is_timedelta OK")
