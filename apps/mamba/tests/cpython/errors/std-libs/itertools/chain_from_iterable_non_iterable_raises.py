# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "errors"
# case = "chain_from_iterable_non_iterable_raises"
# subject = "itertools.chain.from_iterable"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.chain.from_iterable: chain_from_iterable_non_iterable_raises (errors)."""
import itertools

_raised = False
try:
    list(itertools.chain.from_iterable(123))
except TypeError:
    _raised = True
assert _raised, "chain_from_iterable_non_iterable_raises: expected TypeError"
print("chain_from_iterable_non_iterable_raises OK")
