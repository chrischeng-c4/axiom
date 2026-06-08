# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "errors"
# case = "chain_non_iterable_element_raises"
# subject = "itertools.chain"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.chain: chain_non_iterable_element_raises (errors)."""
import itertools

_raised = False
try:
    list(itertools.chain(2, 3))
except TypeError:
    _raised = True
assert _raised, "chain_non_iterable_element_raises: expected TypeError"
print("chain_non_iterable_element_raises OK")
