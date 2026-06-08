# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "timedelta_mixed_units_normalize"
# subject = "datetime.timedelta"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.timedelta: mixed/negative units normalize down to a single microsecond (the canonical carry-cancellation identity)"""
import datetime

t1 = datetime.timedelta(days=100, weeks=-7, hours=-24 * (100 - 49), minutes=-3,
                        seconds=12, microseconds=(3 * 60 - 12) * 1000000.0 + 1)
assert t1 == datetime.timedelta(microseconds=1), f"normalize = {t1!r}"
print("timedelta_mixed_units_normalize OK")
