# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "errors"
# case = "instantiate_iterator_raises"
# subject = "collections.abc.Iterator"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Iterator: instantiate_iterator_raises (errors)."""
import collections.abc

_raised = False
try:
    collections.abc.Iterator()
except TypeError:
    _raised = True
assert _raised, "instantiate_iterator_raises: expected TypeError"
print("instantiate_iterator_raises OK")
