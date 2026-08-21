# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "errors"
# case = "cycle_non_iterable_raises"
# subject = "itertools.cycle"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.cycle: cycle_non_iterable_raises (errors)."""
import itertools

_raised = False
try:
    itertools.cycle(5)
except TypeError:
    _raised = True
assert _raised, "cycle_non_iterable_raises: expected TypeError"
print("cycle_non_iterable_raises OK")
