# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "errors"
# case = "bad_action_raises"
# subject = "warnings.simplefilter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.simplefilter: bad_action_raises (errors)."""
import warnings

_raised = False
try:
    warnings.simplefilter("not_a_valid_action")
except AssertionError:
    _raised = True
assert _raised, "bad_action_raises: expected AssertionError"
print("bad_action_raises OK")
