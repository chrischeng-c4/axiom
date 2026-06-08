# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "bad_timespec_raises"
# subject = "datetime.time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.time: time.isoformat with an unknown timespec keyword raises ValueError"""
import datetime

full = datetime.time(12, 34, 56, 123456)
_raised = False
try:
    full.isoformat(timespec="monkey")
except ValueError:
    _raised = True
assert _raised, "bad_timespec: expected ValueError"
print("bad_timespec_raises OK")
