# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "timedelta_days_field"
# subject = "datetime.timedelta"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.timedelta: timedelta(days=N).days returns N for N in {7,3,100,0}"""
import datetime

for days in (7, 3, 100, 0):
    assert datetime.timedelta(days=days).days == days, f"days={days!r}"
print("timedelta_days_field OK")
