# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "errors"
# case = "cycle_no_argument_raises"
# subject = "itertools.cycle"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.cycle: cycle_no_argument_raises (errors)."""
import itertools

_raised = False
try:
    itertools.cycle()
except TypeError:
    _raised = True
assert _raised, "cycle_no_argument_raises: expected TypeError"
print("cycle_no_argument_raises OK")
