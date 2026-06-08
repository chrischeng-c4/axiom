# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "timedelta_div_by_zero_raises"
# subject = "datetime.timedelta"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.timedelta: true-div, floor-div, and modulo of a timedelta by a zero timedelta each raise ZeroDivisionError"""
import datetime

from operator import truediv, floordiv, mod

t = datetime.timedelta(minutes=2, seconds=30)
zero = datetime.timedelta(0)
for op in (truediv, floordiv, mod):
    _raised = False
    try:
        op(t, zero)
    except ZeroDivisionError:
        _raised = True
    assert _raised, f"{op.__name__} by zero: expected ZeroDivisionError"
print("timedelta_div_by_zero_raises OK")
