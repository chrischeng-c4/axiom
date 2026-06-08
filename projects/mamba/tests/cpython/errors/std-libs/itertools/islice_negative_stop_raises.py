# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "errors"
# case = "islice_negative_stop_raises"
# subject = "itertools.islice"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.islice: islice_negative_stop_raises (errors)."""
import itertools

_raised = False
try:
    list(itertools.islice([1, 2, 3], -1))
except ValueError:
    _raised = True
assert _raised, "islice_negative_stop_raises: expected ValueError"
print("islice_negative_stop_raises OK")
