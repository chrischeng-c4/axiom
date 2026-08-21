# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "timedelta_divmod_by_int_raises"
# subject = "datetime.timedelta"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.timedelta: divmod(timedelta, int) is unsupported (TypeError) even though timedelta / int and timedelta // int are valid"""
import datetime

t = datetime.timedelta(minutes=2, seconds=30)
# timedelta / int is valid (scales the delta).
assert t / 10 == datetime.timedelta(seconds=15), f"td / int = {t / 10!r}"
# divmod against a plain int is unsupported.
_raised = False
try:
    divmod(t, 10)
except TypeError:
    _raised = True
assert _raised, "timedelta_divmod_by_int_raises: expected TypeError"
print("timedelta_divmod_by_int_raises OK")
