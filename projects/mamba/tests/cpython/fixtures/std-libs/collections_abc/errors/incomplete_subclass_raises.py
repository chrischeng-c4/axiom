# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "errors"
# case = "incomplete_subclass_raises"
# subject = "collections.abc.MutableMapping"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.MutableMapping: incomplete_subclass_raises (errors)."""
import collections.abc

_raised = False
try:
    type('IncompleteMap', (collections.abc.MutableMapping,), {})()
except TypeError:
    _raised = True
assert _raised, "incomplete_subclass_raises: expected TypeError"
print("incomplete_subclass_raises OK")
