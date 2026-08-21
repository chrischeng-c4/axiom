# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "errors"
# case = "zip_longest_propagates_source_error"
# subject = "itertools.zip_longest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.zip_longest: zip_longest_propagates_source_error (errors)."""
import itertools

class _R:
    def __init__(self, n):
        self.n = n
    def __iter__(self):
        return self
    def __next__(self):
        if self.n > 0:
            self.n -= 1
            return 1
        raise RuntimeError('boom')

_raised = False
try:
    list(itertools.zip_longest(_R(3), _R(9), fillvalue=0))
except RuntimeError:
    _raised = True
assert _raised, "zip_longest_propagates_source_error: expected RuntimeError"
print("zip_longest_propagates_source_error OK")
