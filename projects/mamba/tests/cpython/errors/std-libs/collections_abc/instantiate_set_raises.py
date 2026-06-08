# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "errors"
# case = "instantiate_set_raises"
# subject = "collections.abc.Set"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Set: instantiate_set_raises (errors)."""
import collections.abc

_raised = False
try:
    collections.abc.Set()
except TypeError:
    _raised = True
assert _raised, "instantiate_set_raises: expected TypeError"
print("instantiate_set_raises OK")
