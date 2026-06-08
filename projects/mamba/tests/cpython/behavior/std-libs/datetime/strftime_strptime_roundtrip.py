# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "strftime_strptime_roundtrip"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.datetime: strftime('%Y-%m-%d %H:%M:%S') renders the expected text and strptime parses it back to the same datetime"""
import datetime

dt = datetime.datetime(2023, 6, 15, 9, 5, 3)
s = dt.strftime("%Y-%m-%d %H:%M:%S")
assert s == "2023-06-15 09:05:03", f"strftime = {s!r}"
parsed = datetime.datetime.strptime(s, "%Y-%m-%d %H:%M:%S")
assert parsed == dt, f"strptime round-trip = {parsed!r}"
print("strftime_strptime_roundtrip OK")
