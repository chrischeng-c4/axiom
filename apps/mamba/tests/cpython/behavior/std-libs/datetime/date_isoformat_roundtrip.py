# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "date_isoformat_roundtrip"
# subject = "datetime.date"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date: date.isoformat() yields YYYY-MM-DD and date.fromisoformat round-trips it back"""
import datetime

d = datetime.date(2023, 6, 15)
assert d.isoformat() == "2023-06-15", f"date iso = {d.isoformat()!r}"
assert datetime.date.fromisoformat(d.isoformat()) == d, "date iso round-trip"
print("date_isoformat_roundtrip OK")
