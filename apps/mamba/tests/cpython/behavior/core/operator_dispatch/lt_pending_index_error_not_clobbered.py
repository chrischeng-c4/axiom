# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "operator_dispatch"
# dimension = "behavior"
# case = "lt_pending_index_error_not_clobbered"
# subject = "binary < pending-exception propagation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A `<` right operand whose own evaluation already raised must propagate
that original exception, not a fresh unorderable-types TypeError from the
comparison dispatcher's final fallback (#1962, same pending-exception guard
shape as #1547 mb_value_cmp / #1938 mb_add — here it guards
`values_lt_fallback`, the shared tail `mb_lt` falls through to and that
`mb_gt`/`mb_le`/`mb_ge` also compose through via `mb_lt(b, a)`).

The left operand `5` evaluates successfully; indexing the empty tuple
`args[0]` then raises IndexError while evaluating the right-hand operand.
"""
args = ()

try:
    5 < args[0]
    raise AssertionError("expected IndexError")
except IndexError:
    pass

print("lt_pending_index_error_not_clobbered OK")
