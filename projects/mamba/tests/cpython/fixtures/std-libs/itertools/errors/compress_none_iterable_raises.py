# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "errors"
# case = "compress_none_iterable_raises"
# subject = "itertools.compress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.compress: compress_none_iterable_raises (errors)."""
import itertools

_raised = False
try:
    itertools.compress(None, range(6))
except TypeError:
    _raised = True
assert _raised, "compress_none_iterable_raises: expected TypeError"
print("compress_none_iterable_raises OK")
