# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "errors"
# case = "instantiate_mapping_raises"
# subject = "collections.abc.Mapping"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Mapping: instantiate_mapping_raises (errors)."""
import collections.abc

_raised = False
try:
    collections.abc.Mapping()
except TypeError:
    _raised = True
assert _raised, "instantiate_mapping_raises: expected TypeError"
print("instantiate_mapping_raises OK")
