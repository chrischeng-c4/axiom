# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "operator_dispatch"
# dimension = "behavior"
# case = "pow_pending_index_error_not_clobbered"
# subject = "binary ** pending-exception propagation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A `**` left operand whose own evaluation already raised must propagate
that original exception, not a fresh operand-type TypeError from the `**`
dispatcher's final catch-all tail (#1962, same pending-exception guard
shape as #1547 mb_value_cmp / #1938 mb_add — here it guards `mb_pow`'s
unsupported-operand-pair raise instead).

Indexing the empty tuple `args[0]` raises IndexError before `**` evaluates
the right-hand `args[1]` subscript.
"""
args = ()

try:
    args[0] ** args[1]
    raise AssertionError("expected IndexError")
except IndexError:
    pass

print("pow_pending_index_error_not_clobbered OK")
