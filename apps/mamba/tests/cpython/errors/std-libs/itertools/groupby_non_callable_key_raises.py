# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "errors"
# case = "groupby_non_callable_key_raises"
# subject = "itertools.groupby"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.groupby: groupby_non_callable_key_raises (errors)."""
import itertools

_raised = False
try:
    list(itertools.groupby('abc', []))
except TypeError:
    _raised = True
assert _raised, "groupby_non_callable_key_raises: expected TypeError"
print("groupby_non_callable_key_raises OK")
