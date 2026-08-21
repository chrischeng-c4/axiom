# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "timezone_dst_non_datetime_raises"
# subject = "datetime.timezone"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.timezone: timezone.dst() applied to a non-datetime argument (str / int) raises TypeError"""
import datetime

EST = datetime.timezone(-datetime.timedelta(hours=5), "EST")
for bad in ("", 5):
    _raised = False
    try:
        EST.dst(bad)
    except TypeError:
        _raised = True
    assert _raised, f"dst({bad!r}): expected TypeError"
print("timezone_dst_non_datetime_raises OK")
