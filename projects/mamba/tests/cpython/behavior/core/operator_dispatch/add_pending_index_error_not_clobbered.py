# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "operator_dispatch"
# dimension = "behavior"
# case = "add_pending_index_error_not_clobbered"
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

Indexing the empty tuple `args[0]` raises IndexError before `+` evaluates
the right-hand `args[1]` subscript.
"""
args = ()

try:
    args[0] + args[1]
    raise AssertionError("expected IndexError")
except IndexError:
    pass

print("add_pending_index_error_not_clobbered OK")
