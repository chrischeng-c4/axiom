# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "timedelta_mixed_int_ops_raise"
# subject = "datetime.timedelta"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.timedelta: timedelta +/- int (either operand order) raises TypeError; int // timedelta raises TypeError; timedelta // 0 and timedelta / 0.0 raise ZeroDivisionError"""
import datetime

a = datetime.timedelta(42)
for value in (1, 1.0):
    for expr in (lambda: a + value, lambda: value + a,
                 lambda: a - value, lambda: value - a):
        _raised = False
        try:
            expr()
        except TypeError:
            _raised = True
        assert _raised, f"mixed {value!r}: expected TypeError"

# int // timedelta is unsupported (TypeError).
_raised = False
try:
    0 // a
except TypeError:
    _raised = True
assert _raised, "int // timedelta: expected TypeError"

# timedelta // 0 and timedelta / 0.0 are ZeroDivisionError.
_raised = False
try:
    a // 0
except ZeroDivisionError:
    _raised = True
assert _raised, "td // 0: expected ZeroDivisionError"
_raised = False
try:
    a / 0.0
except ZeroDivisionError:
    _raised = True
assert _raised, "td / 0.0: expected ZeroDivisionError"
print("timedelta_mixed_int_ops_raise OK")
