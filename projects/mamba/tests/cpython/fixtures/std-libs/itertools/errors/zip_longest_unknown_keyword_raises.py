# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "errors"
# case = "zip_longest_unknown_keyword_raises"
# subject = "itertools.zip_longest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.zip_longest: zip_longest_unknown_keyword_raises (errors)."""
import itertools

_raised = False
try:
    itertools.zip_longest('abc', fillvalue=1, bogus=None)
except TypeError:
    _raised = True
assert _raised, "zip_longest_unknown_keyword_raises: expected TypeError"
print("zip_longest_unknown_keyword_raises OK")
