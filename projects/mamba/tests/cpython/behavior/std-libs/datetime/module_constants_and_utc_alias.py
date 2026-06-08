# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "module_constants_and_utc_alias"
# subject = "datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime: datetime.MINYEAR==1, datetime.MAXYEAR==9999, and datetime.UTC is timezone.utc"""
import datetime

assert datetime.MINYEAR == 1, f"MINYEAR = {datetime.MINYEAR!r}"
assert datetime.MAXYEAR == 9999, f"MAXYEAR = {datetime.MAXYEAR!r}"
assert datetime.UTC is datetime.timezone.utc, "UTC is timezone.utc"
print("module_constants_and_utc_alias OK")
