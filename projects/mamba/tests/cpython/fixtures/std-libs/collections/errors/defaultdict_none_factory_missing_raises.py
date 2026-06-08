# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "errors"
# case = "defaultdict_none_factory_missing_raises"
# subject = "collections.defaultdict"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.defaultdict: defaultdict_none_factory_missing_raises (errors)."""
import collections

_raised = False
try:
    collections.defaultdict(None)['x']
except KeyError:
    _raised = True
assert _raised, "defaultdict_none_factory_missing_raises: expected KeyError"
print("defaultdict_none_factory_missing_raises OK")
