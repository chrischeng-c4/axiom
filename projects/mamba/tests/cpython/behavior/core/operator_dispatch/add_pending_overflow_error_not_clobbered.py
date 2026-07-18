# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "operator_dispatch"
# dimension = "behavior"
# case = "add_pending_overflow_error_not_clobbered"
# subject = "binary + pending-exception propagation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A `+` left operand whose own evaluation already raised must propagate
that original exception, not a fresh operand-type TypeError from the `+`
dispatcher itself (#1962, uncovered while making #1938's None-operand
check strict: the dispatcher's own pending-exception guard, mirroring the
#1547 mb_value_cmp precedent, is what keeps this case correct).

`datetime.timedelta(days=10**10)` overflows during construction, so the
left operand's own evaluation raises OverflowError before `+` evaluates
the right-hand `timedelta(...)` constructor or inspects either result.
"""
import datetime

try:
    datetime.timedelta(days=10**10) + datetime.timedelta(days=10**10)
    raise AssertionError("expected OverflowError")
except OverflowError:
    pass

print("add_pending_overflow_error_not_clobbered OK")
