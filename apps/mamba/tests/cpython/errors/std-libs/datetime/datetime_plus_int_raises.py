# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "datetime_plus_int_raises"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.datetime: datetime + int, int + datetime, datetime * timedelta, and datetime + datetime are each rejected with TypeError"""
import datetime

a = datetime.datetime(2002, 3, 2, 17, 6)
for expr in (lambda: a + 1, lambda: 1 + a,
             lambda: a * datetime.timedelta(1), lambda: a + a):
    _raised = False
    try:
        expr()
    except TypeError:
        _raised = True
    assert _raised, "datetime_plus_int_raises: expected TypeError"
print("datetime_plus_int_raises OK")
