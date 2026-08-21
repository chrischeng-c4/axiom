# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "aware_timestamp_offset"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.datetime: an aware datetime's POSIX timestamp accounts for the offset: 1970-01-01 UTC is 0.0, and an EST-5 instant adds the 5h offset plus its sub-second part"""
import datetime

t = datetime.datetime(1970, 1, 1, tzinfo=datetime.timezone.utc)
assert t.timestamp() == 0.0, f"epoch timestamp = {t.timestamp()!r}"
est = datetime.datetime(1970, 1, 1, 1, 2, 3, 4,
                        tzinfo=datetime.timezone(datetime.timedelta(hours=-5)))
assert est.timestamp() == 18000 + 3600 + 2 * 60 + 3 + 4e-06, f"EST ts = {est.timestamp()!r}"
print("aware_timestamp_offset OK")
