# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "operator_dispatch"
# dimension = "behavior"
# case = "mul_pending_index_error_not_clobbered"
# subject = "binary * pending-exception propagation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A `*` right operand whose own evaluation already raised must propagate
that original exception, not a fresh "can't multiply sequence" TypeError
from the `*` dispatcher's sequence-repeat tail (#1962, same pending-
exception guard shape as #1547 mb_value_cmp / #1938 mb_add — here it
guards `mb_mul`'s non-int-repeat-count raise instead).

Indexing the empty tuple `args[0]` raises IndexError while evaluating the
right-hand operand of `[1, 2, 3] * args[0]`, after the list literal on the
left has already evaluated successfully.
"""
args = ()

try:
    [1, 2, 3] * args[0]
    raise AssertionError("expected IndexError")
except IndexError:
    pass

print("mul_pending_index_error_not_clobbered OK")
