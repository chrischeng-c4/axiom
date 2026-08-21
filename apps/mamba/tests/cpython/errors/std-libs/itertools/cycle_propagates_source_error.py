# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "errors"
# case = "cycle_propagates_source_error"
# subject = "itertools.cycle"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.cycle: cycle_propagates_source_error (errors)."""
import itertools

def _boom():
    yield 1
    raise ValueError('boom')

_raised = False
try:
    list(itertools.cycle(_boom()))
except ValueError:
    _raised = True
assert _raised, "cycle_propagates_source_error: expected ValueError"
print("cycle_propagates_source_error OK")
