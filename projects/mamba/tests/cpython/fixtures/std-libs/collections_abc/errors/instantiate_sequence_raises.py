# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "errors"
# case = "instantiate_sequence_raises"
# subject = "collections.abc.Sequence"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Sequence: instantiate_sequence_raises (errors)."""
import collections.abc

_raised = False
try:
    collections.abc.Sequence()
except TypeError:
    _raised = True
assert _raised, "instantiate_sequence_raises: expected TypeError"
print("instantiate_sequence_raises OK")
