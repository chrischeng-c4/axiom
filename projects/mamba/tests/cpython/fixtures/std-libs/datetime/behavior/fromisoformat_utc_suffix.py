# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "fromisoformat_utc_suffix"
# subject = "datetime.datetime.fromisoformat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.datetime.fromisoformat: datetime.fromisoformat('...+00:00') resolves the tzinfo to timezone.utc"""
import datetime

dt = datetime.datetime.fromisoformat("2014-04-19T13:21:13+00:00")
assert dt.tzinfo is datetime.timezone.utc, f"utc suffix = {dt.tzinfo!r}"
print("fromisoformat_utc_suffix OK")
