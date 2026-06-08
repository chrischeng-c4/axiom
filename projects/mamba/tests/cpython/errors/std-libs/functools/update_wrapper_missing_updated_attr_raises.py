# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "errors"
# case = "update_wrapper_missing_updated_attr_raises"
# subject = "functools.update_wrapper"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.update_wrapper: update_wrapper_missing_updated_attr_raises (errors)."""
import functools

_raised = False
try:
    functools.update_wrapper(lambda: 0, lambda: 0, assigned=("attr",), updated=("missing_d",))
except AttributeError:
    _raised = True
assert _raised, "update_wrapper_missing_updated_attr_raises: expected AttributeError"
print("update_wrapper_missing_updated_attr_raises OK")
