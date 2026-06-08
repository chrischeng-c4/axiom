# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "timedelta_bounds_overflow_raises"
# subject = "datetime.timedelta"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.timedelta: stepping one resolution past timedelta.min/.max, negating timedelta.max, and scaling a day beyond range (int * and tiny /) each raise OverflowError"""
import datetime

tiny = datetime.timedelta.resolution

# Stepping one resolution past min / max overflows.
_raised = False
try:
    datetime.timedelta.min.__sub__(tiny)
except OverflowError:
    _raised = True
assert _raised, "min - tiny: expected OverflowError"
_raised = False
try:
    datetime.timedelta.max.__add__(tiny)
except OverflowError:
    _raised = True
assert _raised, "max + tiny: expected OverflowError"

# Negating the extreme positive delta overflows the negative range.
_raised = False
try:
    -datetime.timedelta.max
except OverflowError:
    _raised = True
assert _raised, "neg max: expected OverflowError"

# Scaling a single day beyond range overflows (int * and tiny /).
day = datetime.timedelta(1)
_raised = False
try:
    day.__mul__(10 ** 9)
except OverflowError:
    _raised = True
assert _raised, "day * 1e9: expected OverflowError"
_raised = False
try:
    day.__truediv__(1e-20)
except OverflowError:
    _raised = True
assert _raised, "day / 1e-20: expected OverflowError"
print("timedelta_bounds_overflow_raises OK")
