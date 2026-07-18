# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "operator_dispatch"
# dimension = "behavior"
# case = "truediv_pending_index_error_not_clobbered"
# subject = "binary / pending-exception propagation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A `/` right operand whose own evaluation already raised must propagate
that original exception, not a fresh operand-type TypeError from the
datetime-arithmetic operand-mismatch check shared by `+ - * / % //`
(#1962, same pending-exception guard shape as #1547 mb_value_cmp / #1938
mb_add — here it guards `raise_datetime_op_type_error`, which fires
whenever one operand is a `datetime.*` instance regardless of whether the
other operand is a real value or the `None` sentinel a raise leaves
behind).

The left operand `datetime.timedelta(days=1)` evaluates successfully;
indexing the empty tuple `args[0]` then raises IndexError while
evaluating the right-hand operand.
"""
import datetime

args = ()

try:
    datetime.timedelta(days=1) / args[0]
    raise AssertionError("expected IndexError")
except IndexError:
    pass

print("truediv_pending_index_error_not_clobbered OK")
