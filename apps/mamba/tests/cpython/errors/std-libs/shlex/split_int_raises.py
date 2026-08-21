# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "errors"
# case = "split_int_raises"
# subject = "shlex.split"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shlex.py"
# status = "filled"
# ///
"""shlex.split: split_int_raises (errors)."""
import shlex

_raised = False
try:
    shlex.split(123)
except AttributeError:
    _raised = True
assert _raised, "split_int_raises: expected AttributeError"
print("split_int_raises OK")
