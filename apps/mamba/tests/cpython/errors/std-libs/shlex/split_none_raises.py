# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "errors"
# case = "split_none_raises"
# subject = "shlex.split"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shlex.py"
# status = "filled"
# ///
"""shlex.split: split_none_raises (errors)."""
import shlex

_raised = False
try:
    shlex.split(None)
except ValueError:
    _raised = True
assert _raised, "split_none_raises: expected ValueError"
print("split_none_raises OK")
