# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "errors"
# case = "chain_propagates_source_error"
# subject = "itertools.chain"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.chain: chain_propagates_source_error (errors)."""
import itertools

def _boom():
    yield 1
    raise ValueError('boom')

_raised = False
try:
    list(itertools.chain(_boom(), [9, 9]))
except ValueError:
    _raised = True
assert _raised, "chain_propagates_source_error: expected ValueError"
print("chain_propagates_source_error OK")
