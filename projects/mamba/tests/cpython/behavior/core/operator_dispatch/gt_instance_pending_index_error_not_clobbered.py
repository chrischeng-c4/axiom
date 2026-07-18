# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "operator_dispatch"
# dimension = "behavior"
# case = "gt_instance_pending_index_error_not_clobbered"
# subject = "binary > pending-exception propagation (Instance operand)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A `>` right operand whose own evaluation already raised must propagate
that original exception, not a fresh unorderable-types TypeError from
`unsupported_ordering_bool` (#1962, same pending-exception guard shape as
#1547 mb_value_cmp / #1938 mb_add). When the left operand is a real
`Instance` without `__gt__` (and the right operand is the `None` sentinel a
raise left behind), `mb_gt` reaches `unsupported_ordering_bool` directly
instead of composing through `mb_lt`/`values_lt_fallback`, so this is a
distinct raise site from the plain `<` case.

The left operand `Plain()` evaluates successfully; indexing the empty tuple
`args[0]` then raises IndexError while evaluating the right-hand operand.
"""


class Plain:
    pass


args = ()

try:
    Plain() > args[0]
    raise AssertionError("expected IndexError")
except IndexError:
    pass

print("gt_instance_pending_index_error_not_clobbered OK")
