# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "errors"
# case = "total_ordering_no_op_raises"
# subject = "functools.total_ordering"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.total_ordering: total_ordering_no_op_raises (errors)."""
import functools

_raised = False
try:
    functools.total_ordering(type("E", (), {}))
except ValueError:
    _raised = True
assert _raised, "total_ordering_no_op_raises: expected ValueError"
print("total_ordering_no_op_raises OK")
